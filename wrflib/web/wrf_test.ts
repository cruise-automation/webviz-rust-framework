// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import { CallRust } from "./types";
import { jsRuntime } from "./type_of_runtime";
import { allocatedArcs, allocatedVecs, WrfBuffer } from "./wrf_buffer";

export const expect = <T>(actual: T, expected: T): void => {
  if (expected === actual) {
    console.debug(`Success: Got ${actual}, Expected ${expected}`);
  } else {
    throw new Error(`Failure: Got ${actual}, Expected ${expected}`);
  }
};

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const checkConditionTimeout = async (
  condition: () => boolean,
  timeout: number
) => {
  const startTime = performance.now();
  while (!condition() && performance.now() < startTime + timeout) {
    await sleep(10);
  }
  return condition();
};

const arcDeallocated = async (callRust: CallRust, arcPtr: number) => {
  expect(allocatedArcs[arcPtr], true);

  const [result] = await callRust("check_arc_count", [`${BigInt(arcPtr)}`]);
  const [countBeforeDeallocation] = result;
  expect(countBeforeDeallocation, 1);

  expect(
    await checkConditionTimeout(() => allocatedArcs[arcPtr] === false, 20000),
    true
  );
};

const vecDeallocated = async (bufferPtr: number) => {
  expect(allocatedVecs[bufferPtr], true);

  expect(
    await checkConditionTimeout(
      () => allocatedVecs[bufferPtr] === false,
      20000
    ),
    true
  );
};

// Test that WrfBuffers were deallocated at some point in the next 20 seconds.
// This is a bit brittle given that there are no guarantees for garbage collection during this time,
// but observationally this ends up being enough time. The caller must also ensure that the buffer will go out of scope
// shortly after calling this.
// We have to pass in `callRust` because we can call this function from a variety of runtimes.
export const expectDeallocation = (
  callRust: CallRust,
  wrfArray: Uint8Array
): Promise<void> => {
  // Deallocation code is only run in WASM for now.
  if (jsRuntime === "cef") return Promise.resolve();

  const buffer = wrfArray.buffer as WrfBuffer;
  return buffer.readonly
    ? arcDeallocated(callRust, buffer.__wrflibBufferData.arcPtr)
    : vecDeallocated(buffer.__wrflibBufferData.bufferPtr);
};

export let inTest = false;
// Set this to true to enable testing code
export const setInTest = (v: boolean): void => {
  inTest = v;
};
