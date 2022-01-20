// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

/// <reference lib="WebWorker" />

// The "Wrflib WebWorker runtime" exposes some common Wrflib functions inside your WebWorkers, like `callRust`.
//
// Include the output of this (wrflib_worker_runtime.js) at the start of each worker, and initialize the runtime
// by calling `self.initializeWorker` with a `MessagePort` obtained by `newWorkerPort` (which is
// available on `window` in the main browser thread, and in any worker that already has the runtime running). You
// can pass the port to the worker using `postMessage`; just be sure to include it in the list of transferables.
//
// Currently this is only supported in WebAssembly, not when using CEF.

import {
  createWasmBuffer,
  getWasmEnv,
  getWrfParamType,
  initThreadLocalStorageAndStackOtherWorkers,
  makeZerdeBuilder,
  Rpc,
  transformParamsFromRustImpl,
} from "./common";
import { MainWorkerChannelEvent, WebWorkerRpc } from "./rpc_types";
import {
  CallRust,
  CallRustInSameThreadSync,
  PostMessageTypedArray,
  WasmExports,
  WrfParamType,
  WrfArray,
  RustWrfParam,
  MutableBufferData,
  CreateBufferWorkerSync,
} from "./types";
import { inWorker } from "./type_of_runtime";
import {
  getWrfBufferWasm,
  isWrfBuffer,
  overwriteTypedArraysWithWrfArrays,
  unregisterMutableBuffer,
  WrfBuffer,
  checkValidWrfArray,
} from "./wrf_buffer";
import { ZerdeParser } from "./zerde";

let rpc: Rpc<WebWorkerRpc>;
let wasmExports: WasmExports;
let wasmMemory: WebAssembly.Memory;
let wasmAppPtr: BigInt;

let alreadyCalledInitialize = false;
export const initializeWorker = (wrfWorkerPort: MessagePort): Promise<void> => {
  if (alreadyCalledInitialize) {
    throw new Error("Only call wrflib.initializeWorker once");
  }
  alreadyCalledInitialize = true;

  if (!inWorker) {
    throw new Error(
      "wrflib.initializeWorker() can only be called in a WebWorker"
    );
  }

  overwriteTypedArraysWithWrfArrays();

  return new Promise((resolve) => {
    rpc = new Rpc(wrfWorkerPort);

    rpc
      .send(MainWorkerChannelEvent.Init)
      .then(
        ({
          wasmModule,
          memory,
          taskWorkerSab,
          baseUri,
          appPtr,
          tlsAndStackData,
        }) => {
          wasmMemory = memory;
          wasmAppPtr = appPtr;

          function getExports() {
            return wasmExports;
          }

          const env = getWasmEnv({
            getExports,
            memory,
            taskWorkerSab,
            fileHandles: [], // TODO(JP): implement at some point..
            sendEventFromAnyThread: (eventPtr: BigInt) => {
              rpc.send(MainWorkerChannelEvent.SendEventFromAnyThread, eventPtr);
            },
            threadSpawn: () => {
              throw new Error("Not yet implemented");
            },
            baseUri,
          });

          WebAssembly.instantiate(wasmModule, { env }).then((instance: any) => {
            wasmExports = instance.exports;
            initThreadLocalStorageAndStackOtherWorkers(
              wasmExports,
              tlsAndStackData
            );

            resolve();
          });
        }
      );
  });
};

const destructor = (arcPtr: number) => {
  wasmExports.decrementArc(BigInt(arcPtr));
};

const mutableDestructor = ({
  bufferPtr,
  bufferLen,
  bufferCap,
}: MutableBufferData) => {
  wasmExports.deallocVec(
    BigInt(bufferPtr),
    BigInt(bufferLen),
    BigInt(bufferCap)
  );
};

const transformParamsFromRust = (params: RustWrfParam[]) =>
  transformParamsFromRustImpl(
    wasmMemory,
    destructor,
    mutableDestructor,
    params
  );

export const newWorkerPort = (): MessagePort => {
  const channel = new MessageChannel();
  rpc.send(MainWorkerChannelEvent.BindMainWorkerPort, channel.port1, [
    channel.port1,
  ]);
  return channel.port2;
};

// TODO(JP): Allocate buffers on the wasm memory directly here.
export const callRust: CallRust = async (name, params = []) => {
  const transformedParams = params.map((param) => {
    if (typeof param === "string") {
      return param;
    } else if (isWrfBuffer(param.buffer)) {
      checkValidWrfArray(param);
      return serializeWrfArrayForPostMessage(param);
    } else {
      if (!(param.buffer instanceof SharedArrayBuffer)) {
        console.warn(
          "Consider passing Uint8Arrays backed by WrfBuffer or SharedArrayBuffer into `callRust` to prevent copying data"
        );
      }
      return param;
    }
  });

  return transformParamsFromRust(
    await rpc.send(MainWorkerChannelEvent.CallRust, {
      name,
      params: transformedParams,
    })
  );
};

// TODO(JP): Some of this code is duplicated with callRust/call_js; see if we can reuse some.
export const callRustInSameThreadSync: CallRustInSameThreadSync = (
  name,
  params = []
) => {
  const zerdeBuilder = makeZerdeBuilder(wasmMemory, wasmExports);
  zerdeBuilder.sendString(name);
  zerdeBuilder.sendU32(params.length);
  for (const param of params) {
    if (typeof param === "string") {
      zerdeBuilder.sendU32(WrfParamType.String);
      zerdeBuilder.sendString(param);
    } else {
      if (param.buffer instanceof WrfBuffer) {
        checkValidWrfArray(param);
        if (param.buffer.__wrflibBufferData.readonly) {
          zerdeBuilder.sendU32(getWrfParamType(param, true));

          const arcPtr = param.buffer.__wrflibBufferData.arcPtr;

          // WrfParam parsing code will construct an Arc without incrementing
          // the count, so we do it here ahead of time.
          wasmExports.incrementArc(BigInt(arcPtr));
          zerdeBuilder.sendU32(arcPtr);
        } else {
          // TODO(Paras): User should not be able to access the buffer after
          // passing it to Rust here
          unregisterMutableBuffer(param.buffer);
          zerdeBuilder.sendU32(getWrfParamType(param, false));
          zerdeBuilder.sendU32(param.buffer.__wrflibBufferData.bufferPtr);
          zerdeBuilder.sendU32(param.buffer.__wrflibBufferData.bufferLen);
          zerdeBuilder.sendU32(param.buffer.__wrflibBufferData.bufferCap);
        }
      } else {
        console.warn(
          "Consider passing Uint8Arrays backed by WrfBuffer to prevent copying data"
        );

        const vecLen = param.byteLength;
        const vecPtr = createWasmBuffer(wasmMemory, wasmExports, param);
        zerdeBuilder.sendU32(getWrfParamType(param, false));
        zerdeBuilder.sendU32(vecPtr);
        zerdeBuilder.sendU32(vecLen);
        zerdeBuilder.sendU32(vecLen);
      }
    }
  }
  const returnPtr = wasmExports.callRustInSameThreadSync(
    wasmAppPtr,
    BigInt(zerdeBuilder.getData().byteOffset)
  );

  const zerdeParser = new ZerdeParser(wasmMemory, Number(returnPtr));
  const returnParams = zerdeParser.parseWrfParams();
  return transformParamsFromRust(returnParams);
};

// TODO(JP): See comment at CreateBufferWorkerSync type.
export const createMutableBuffer: CreateBufferWorkerSync = (data) => {
  const bufferLen = data.byteLength;
  const bufferPtr = createWasmBuffer(wasmMemory, wasmExports, data);
  return transformParamsFromRust([
    {
      paramType: getWrfParamType(data, false),
      bufferPtr,
      bufferLen,
      bufferCap: bufferLen,
      readonly: false,
    },
  ])[0] as typeof data;
};

// TODO(JP): See comment at CreateBufferWorkerSync type.
export const createReadOnlyBuffer: CreateBufferWorkerSync = (data) => {
  const bufferPtr = createWasmBuffer(wasmMemory, wasmExports, data);
  const paramType = getWrfParamType(data, true);
  const arcPtr = Number(
    wasmExports.createArcVec(
      BigInt(bufferPtr),
      BigInt(data.length),
      BigInt(paramType)
    )
  );

  return transformParamsFromRust([
    {
      paramType,
      bufferPtr,
      bufferLen: data.byteLength,
      arcPtr,
      readonly: true,
    },
  ])[0] as typeof data;
};

// TODO(JP): Somewhat duplicated with the other implementation.
export const serializeWrfArrayForPostMessage = (
  wrfArray: WrfArray
): PostMessageTypedArray => {
  if (!(typeof wrfArray === "object" && isWrfBuffer(wrfArray.buffer))) {
    throw new Error("Only pass Wrf arrays to serializeWrfArrayForPostMessage");
  }
  const wrfBuffer = wrfArray.buffer as WrfBuffer;
  if (wrfBuffer.__wrflibBufferData.readonly) {
    wasmExports.incrementArc(BigInt(wrfBuffer.__wrflibBufferData.arcPtr));
  } else {
    unregisterMutableBuffer(wrfBuffer);
  }
  return {
    bufferData: wrfBuffer.__wrflibBufferData,
    byteOffset: wrfArray.byteOffset,
    byteLength: wrfArray.byteLength,
  };
};

export const deserializeWrfArrayFromPostMessage = (
  postMessageData: PostMessageTypedArray
): Uint8Array => {
  const wrfBuffer = getWrfBufferWasm(
    wasmMemory,
    postMessageData.bufferData,
    destructor,
    mutableDestructor
  );
  return new Uint8Array(
    wrfBuffer,
    postMessageData.byteOffset,
    postMessageData.byteLength
  );
};
