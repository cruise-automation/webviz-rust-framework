// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import { WrfParamType } from "./types";
import { WrfBuffer, classesToExtend, containsWrfBuffer } from "./wrf_buffer";
import { expect, expectThrow } from "./test_helpers";

declare global {
  interface Window {
    WrfUint8Array: typeof Uint8Array;
    WrfUint16Array: typeof Uint16Array;
  }
}

const { WrfUint8Array, WrfUint16Array } = window;

// Test that WrfArray is created like a DataView
function testBuffer(): void {
  const wasmMemory = new SharedArrayBuffer(1024);
  const buffer = new WrfBuffer(wasmMemory, {
    bufferPtr: 10,
    bufferLen: 4,
    bufferCap: 4,
    paramType: WrfParamType.U8Buffer,
    readonly: false,
  });
  const a = new WrfUint8Array(buffer, 10, 4);
  expect(a.byteOffset, 10);
  expect(a.length, 4);
}

// Test that new WrfArray shares the same WrfBuffer
function testShare(): void {
  const wasmMemory = new SharedArrayBuffer(1024);
  const buffer = new WrfBuffer(wasmMemory, {
    bufferPtr: 0,
    bufferLen: 1024,
    bufferCap: 1024,
    paramType: WrfParamType.U8Buffer,
    readonly: false,
  });
  const a = new WrfUint8Array(buffer);
  const b = new WrfUint16Array(a.buffer);
  expect(a.buffer, buffer);
  expect(a.buffer, b.buffer);
}

// Test WrfArray out-of-bounds behavior
function testOutOfBounds(): void {
  const wasmMemory = new SharedArrayBuffer(1024);
  const buffer = new WrfBuffer(wasmMemory, {
    bufferPtr: 1,
    bufferLen: 16,
    bufferCap: 16,
    paramType: WrfParamType.U8Buffer,
    readonly: false,
  });
  // start is outside of the view - should throw
  expectThrow(() => {
    new WrfUint8Array(buffer, 0);
  }, "Byte_offset 0 is out of bounds");

  // these doesn't throw but overwrites the end of the data
  const a = new WrfUint8Array(buffer, 1);
  expect(a.length, 16);
  const b = new WrfUint8Array(buffer, 2);
  expect(b.length, 15);

  // end is outside of the view - should throw
  expectThrow(() => {
    new WrfUint8Array(buffer, 15, 3);
  }, "Byte_offset 15 + length 3 is out of bounds");
}

// Test that WrfBuffer and WrfArray could be created from ArrayBuffer
function testArrayBuffer(): void {
  const array = new ArrayBuffer(16);
  const buffer = new WrfBuffer(array, {
    bufferPtr: 0,
    bufferLen: array.byteLength,
    bufferCap: array.byteLength,
    paramType: WrfParamType.U8Buffer,
    readonly: false,
  });
  const a = new WrfUint8Array(buffer);
  expect(a.byteOffset, 0);
  expect(a.byteLength, 16);
}

// Check that all names follow the convetion of having Wrf as prefix
// e.g. WrfUint8Array overrides Uint8Array
function testWrfNameMatches(): void {
  for (const [cls, wrfCls] of Object.entries(classesToExtend)) {
    const expectedName = "Wrf" + cls;
    expect(expectedName, wrfCls);
  }
}

function testSubarray(): void {
  const wasmMemory = new SharedArrayBuffer(5);
  const regularArray = new Uint8Array(wasmMemory);
  regularArray.set(Uint8Array.from([0, 1, 2, 3, 4]));
  const buffer = new WrfBuffer(wasmMemory, {
    bufferPtr: 0,
    bufferLen: 5,
    bufferCap: 5,
    paramType: WrfParamType.U8Buffer,
    readonly: false,
  });
  const wrfArray = new WrfUint8Array(buffer);

  expect(wrfArray.subarray().buffer, buffer);
  expect(wrfArray.subarray().toString(), regularArray.subarray().toString());
  expect(
    wrfArray.subarray(1, 3).toString(),
    regularArray.subarray(1, 3).toString()
  );
  expect(
    wrfArray.subarray(-2, 0).toString(),
    regularArray.subarray(-2, 0).toString()
  );
  expect(
    wrfArray.subarray(-3, -1).toString(),
    regularArray.subarray(-3, -1).toString()
  );
  expect(
    wrfArray.subarray(1, -1).toString(),
    regularArray.subarray(1, -1).toString()
  );
}

function testContainsWrfBuffer(): void {
  const wasmMemory = new SharedArrayBuffer(16);
  const buffer = new WrfBuffer(wasmMemory, {
    bufferPtr: 0,
    bufferLen: 16,
    bufferCap: 16,
    paramType: WrfParamType.U8Buffer,
    readonly: false,
  });
  const a = new WrfUint8Array(buffer);

  expect(containsWrfBuffer(a), true);
  expect(containsWrfBuffer([a]), true);
  expect(containsWrfBuffer({ key: a }), true);
  expect(containsWrfBuffer(new Set([a])), true);

  const map = new Map();
  map.set("key", a);
  expect(containsWrfBuffer(map), true);

  // calling slice removes the error
  expect(containsWrfBuffer(a.slice()), false);

  // edge cases
  expect(containsWrfBuffer(undefined), false);
  expect(containsWrfBuffer(null), false);
}

export const wrfBufferTests = {
  testBuffer,
  testShare,
  testOutOfBounds,
  testWrfNameMatches,
  testArrayBuffer,
  testSubarray,
  testContainsWrfBuffer,
};
