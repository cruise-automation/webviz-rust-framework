// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// Wrapper around SharedArrayBuffer to encapsulate ownership of particular segments of it

import { getWrfParamType } from "./common";
import { BufferData, MutableBufferData, WrfArray, WrfParamType } from "./types";
import { inTest } from "./wrf_test";

// TODO(Paras) - Make sure we monkeypatch on web workers as well
export class WrfBuffer extends SharedArrayBuffer {
  // This class supports both SharedArrayBuffer (wasm usecase) and ArrayBuffer (CEF)
  // In the future we can migrate to SharedArrayBuffer-s only once CEF supports those
  __wrflibWasmBuffer: SharedArrayBuffer | ArrayBuffer;
  __wrflibBufferData: BufferData;

  constructor(buffer: SharedArrayBuffer | ArrayBuffer, bufferData: BufferData) {
    super(0);
    this.__wrflibWasmBuffer = buffer;
    this.__wrflibBufferData = bufferData;
  }

  // TODO(Paras): Actually enforce this flag and prevent mutation of WrfArrays marked as readonly.
  // Potentially, we can do this by hashing read only buffer data and periodically checking in debug
  // builds if they have been modified/raising errors.
  get readonly(): boolean {
    return this.__wrflibBufferData.readonly;
  }

  // The only 2 methods on SharedArrayBuffer class to override:
  // See https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer#instance_properties
  get byteLength(): number {
    return this.__wrflibWasmBuffer.byteLength;
  }

  slice(...args: Parameters<SharedArrayBuffer["slice"]>): any {
    return this.__wrflibWasmBuffer.slice(...args);
  }
}

// This class is a drop-in replacement for all typed arrays
// It uses WrfBuffer as a handle for underlying buffer as the object that keeps underlying data around
// Requirements:
//  * The underlying typed array behaves like it was created over the original view
//  * When the new typed array (potentially with different class name) is created from the buffer of the original one,
//  they share the same handle
//
// The Rust side assumes that underlying data buffer is immutable,
// however it still could be accidentally modified on JS side leading to weird behavior
// TODO(Dmitry): Throw an error if there is mutation of the data
function wrfBufferExtends(cls: any) {
  return class WrfTypedArray extends cls {
    constructor(...args: any) {
      const buffer = args[0];
      if (typeof buffer === "object" && buffer instanceof WrfBuffer) {
        // Fill in byteOffset if that's omitted.
        if (args.length < 2) {
          args[1] = buffer.__wrflibBufferData.bufferPtr;
        }
        // Fill in length (in elements, not in bytes) if that's omitted.
        if (args.length < 3) {
          args[2] = Math.floor(
            (buffer.__wrflibBufferData.bufferPtr +
              buffer.__wrflibBufferData.bufferLen -
              args[1]) /
              cls.BYTES_PER_ELEMENT
          );
        }
        if (args[1] < buffer.__wrflibBufferData.bufferPtr) {
          throw new Error(`Byte_offset ${args[1]} is out of bounds`);
        }
        if (
          args[1] + args[2] * cls.BYTES_PER_ELEMENT >
          buffer.__wrflibBufferData.bufferPtr +
            buffer.__wrflibBufferData.bufferLen
        ) {
          throw new Error(
            `Byte_offset ${args[1]} + length ${args[2]} is out of bounds`
          );
        }
        // Whenever we create WrfUintArray using WrfBuffer as first argument
        // pass the underlying full wasm_buffer further
        args[0] = buffer.__wrflibWasmBuffer;
        super(...args);
        this.__wrflibBuffer = buffer;
      } else {
        super(...args);
      }
    }

    get buffer() {
      return this.__wrflibBuffer || super.buffer;
    }

    subarray(begin = 0, end = this.length) {
      if (begin < 0) {
        begin = this.length + begin;
      }
      if (end < 0) {
        end = this.length + end;
      }
      if (end < begin) {
        end = begin;
      }
      return new WrfTypedArray(
        this.buffer,
        this.byteOffset + begin * this.BYTES_PER_ELEMENT,
        end - begin
      );
    }
  };
}

// Extending all typed arrays
// See https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects#indexed_collections
export const classesToExtend = {
  Int8Array: "WrfInt8Array",
  Uint8Array: "WrfUint8Array",
  Uint8ClampedArray: "WrfUint8ClampedArray",
  Int16Array: "WrfInt16Array",
  Uint16Array: "WrfUint16Array",
  Uint16ClampedArray: "WrfUint16ClampedArray",
  Int32Array: "WrfInt32Array",
  Uint32Array: "WrfUint32Array",
  Float32Array: "WrfFloat32Array",
  Float64Array: "WrfFloat64Array",
  BigInt64Array: "WrfBigInt64Array",
  BigUint64Array: "WrfBigUint64Array",
  DataView: "WrfDataView",
};

for (const [cls, wrfCls] of Object.entries(classesToExtend)) {
  // Get a new type name by prefixing old one with "Wrf".
  // e.g. Uint8Array is extended by WrfUint8Array, etc
  if (cls in self) {
    // @ts-ignore
    self[wrfCls] = wrfBufferExtends(self[cls]);
  }
}

// Checks if the given object itself or recursively contains WrfBuffers.
// Exported for tests.
export function containsWrfBuffer(object: unknown): boolean {
  if (typeof object != "object" || object === null) {
    return false;
  }

  if (Object.prototype.hasOwnProperty.call(object, "__wrflibBuffer")) {
    return true;
  }

  // Only supporting nesting for arrays, plain objects, maps and sets similar to StructuredClone algorithm
  // See https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm#supported_types
  if (Array.isArray(object) || object instanceof Set || object instanceof Map) {
    for (const entry of object) {
      if (containsWrfBuffer(entry)) {
        return true;
      }
    }
  } else if (Object.getPrototypeOf(object) === Object.getPrototypeOf({})) {
    for (const entry of Object.entries(object)) {
      if (containsWrfBuffer(entry)) {
        return true;
      }
    }
  }
  return false;
}

function patchPostMessage(cls: any) {
  const origPostMessage = cls.postMessage;
  // Explicitly NOT a fat arrow (=>) since we want to keep the original `this`.
  cls.postMessage = function (...args: Parameters<Worker["postMessage"]>) {
    if (containsWrfBuffer(args[0])) {
      // TODO(Dmitry): add a better error message showing the exact location of typed arrays
      throw new Error(
        "Sending WrfBuffers to/from workers is not supported - " +
          "use .slice() on typed array instead to make an explicit copy"
      );
    }
    origPostMessage.apply(this, args);
  };
}

export function overwriteTypedArraysWithWrfArrays(): void {
  for (const [cls, wrfCls] of Object.entries(classesToExtend)) {
    if (cls in self) {
      // @ts-ignore
      self[cls] = self[wrfCls];
    }
  }
  patchPostMessage(self);
  patchPostMessage(self.Worker);
  patchPostMessage(self.MessagePort);
}

const wrfBufferCache = new WeakMap<WrfBuffer, WrfArray>();
export function getCachedWrfBuffer(
  wrfBuffer: WrfBuffer,
  fallbackArray: WrfArray
): WrfArray {
  if (
    !(
      // Overwrite the cached value if we return a pointer to a buffer of a different type
      // For example, Rust code may cast a float to an u8 and return the same buffer pointer.
      (
        wrfBufferCache.get(wrfBuffer)?.BYTES_PER_ELEMENT ===
        fallbackArray.BYTES_PER_ELEMENT
      )
    )
  ) {
    wrfBufferCache.set(wrfBuffer, fallbackArray);
  }
  return wrfBufferCache.get(wrfBuffer) as WrfArray;
}

export function isWrfBuffer(potentialWrfBuffer: unknown): boolean {
  return (
    typeof potentialWrfBuffer === "object" &&
    potentialWrfBuffer instanceof WrfBuffer
  );
}

export function checkValidWrfArray(wrfArray: WrfArray): void {
  if (!isWrfBuffer(wrfArray.buffer)) {
    throw new Error("wrfArray.buffer is not a WrfBuffer in checkValidWrfArray");
  }
  const buffer = wrfArray.buffer as WrfBuffer;

  const bufferCoversWrfBuffer =
    wrfArray.byteOffset === buffer.__wrflibBufferData.bufferPtr &&
    wrfArray.byteLength === buffer.__wrflibBufferData.bufferLen;
  if (!bufferCoversWrfBuffer) {
    throw new Error(
      "Called Rust with a buffer that does not span the entire underlying WrfBuffer"
    );
  }

  const paramType = getWrfParamType(wrfArray, buffer.readonly);
  if (paramType !== buffer.__wrflibBufferData.paramType) {
    throw new Error(
      `Cannot call Rust with a buffer which has been cast to a different type. Expected ${
        WrfParamType[buffer.__wrflibBufferData.paramType]
      } but got ${WrfParamType[paramType]}`
    );
  }
}

// Cache WrfBuffers so that we have a stable identity for WrfBuffers pointing to the same
// Arc. This is useful for any downstream caches in user code.
const bufferCache: { [arcPtr: number]: WeakRef<WrfBuffer> } = {};

export const allocatedArcs: Record<number, boolean> = {};
export const allocatedVecs: Record<number, boolean> = {};

const bufferRegistry = new FinalizationRegistry(
  ({
    arcPtr,
    destructor,
  }: {
    arcPtr: number;
    destructor?: (arcPtr: number) => void;
  }) => {
    if (inTest) {
      if (allocatedArcs[arcPtr] === false) {
        throw new Error(`Deallocating an already deallocated arcPtr ${arcPtr}`);
      } else if (allocatedArcs[arcPtr] === undefined) {
        throw new Error(`Deallocating an unallocated arcPtr ${arcPtr}`);
      }
      allocatedArcs[arcPtr] = false;
    }

    delete bufferCache[arcPtr];
    if (destructor) destructor(arcPtr);
  }
);

const mutableWrfBufferRegistry = new FinalizationRegistry(
  ({
    bufferData,
    destructor,
  }: {
    bufferData: MutableBufferData;
    destructor: (bufferData: MutableBufferData) => void;
  }) => {
    if (inTest) {
      const { bufferPtr } = bufferData;
      if (allocatedVecs[bufferPtr] === false) {
        throw new Error(
          `Deallocating an already deallocated bufferPtr ${bufferPtr}`
        );
      } else if (allocatedVecs[bufferPtr] === undefined) {
        throw new Error(`Deallocating an unallocated bufferPtr ${bufferPtr}`);
      }
      allocatedVecs[bufferPtr] = false;
    }

    destructor(bufferData);
  }
);

// Return a buffer with a stable identity based on arcPtr.
// Register callbacks so we de-allocate the buffer when it goes out of scope.
export const getWrfBufferWasm = (
  wasmMemory: WebAssembly.Memory,
  bufferData: BufferData,
  destructor: (arcPtr: number) => void,
  mutableDestructor: (bufferData: MutableBufferData) => void
): WrfBuffer => {
  if (bufferData.readonly) {
    if (!bufferCache[bufferData.arcPtr]?.deref()) {
      if (inTest) {
        allocatedArcs[bufferData.arcPtr] = true;
      }

      const wrfBuffer = new WrfBuffer(wasmMemory.buffer, bufferData);

      bufferRegistry.register(wrfBuffer, {
        arcPtr: bufferData.arcPtr,
        destructor,
        /* no unregisterToken here since we never need to unregister */
      });

      bufferCache[bufferData.arcPtr] = new WeakRef(wrfBuffer);
    } else {
      // If we already hold a reference, decrement the Arc we were just given;
      // otherwise we leak memory.
      destructor(bufferData.arcPtr);
    }

    return bufferCache[bufferData.arcPtr].deref() as WrfBuffer;
  } else {
    if (inTest) {
      allocatedVecs[bufferData.bufferPtr] = true;
    }

    const wrfBuffer = new WrfBuffer(wasmMemory.buffer, bufferData);

    mutableWrfBufferRegistry.register(
      wrfBuffer,
      {
        bufferData,
        destructor: mutableDestructor,
      },
      wrfBuffer
    );

    return wrfBuffer;
  }
};

// Remove mutable WrfBuffers without running destructors. This is useful
// when transferring ownership of buffers to Rust without deallocating data.
export const unregisterMutableBuffer = (wrfBuffer: WrfBuffer): void => {
  if (wrfBuffer.readonly) {
    throw new Error(
      "`unregisterMutableBuffer` should only be called on mutable WrfBuffers"
    );
  }

  mutableWrfBufferRegistry.unregister(wrfBuffer);

  if (inTest) {
    allocatedVecs[wrfBuffer.__wrflibBufferData.bufferPtr] = false;
  }
};

// Return a buffer with a stable identity based on arcPtr
export const getWrfBufferCef = (
  buffer: ArrayBuffer,
  arcPtr: number | undefined,
  paramType: WrfParamType
): WrfBuffer => {
  if (arcPtr) {
    if (!bufferCache[arcPtr]?.deref()) {
      const wrfBuffer = new WrfBuffer(buffer, {
        bufferPtr: 0,
        bufferLen: buffer.byteLength,
        readonly: true,
        paramType,
        // TODO(Paras): These fields below do not apply to CEF
        arcPtr: -1,
      });

      bufferRegistry.register(wrfBuffer, { arcPtr });
      bufferCache[arcPtr] = new WeakRef(wrfBuffer);
    }
    return bufferCache[arcPtr].deref() as WrfBuffer;
  } else {
    return new WrfBuffer(buffer, {
      bufferPtr: 0,
      bufferLen: buffer.byteLength,
      bufferCap: buffer.byteLength,
      paramType,
      readonly: false,
    });
  }
};
