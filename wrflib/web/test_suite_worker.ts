// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import "./wrf_web_worker_runtime";
import { expect } from "./wrf_test";
import { Rpc } from "./common";
import { TestSuiteWorkerSpec } from "./test_suite";
import { Worker } from "./rpc_types";

declare let self: WorkerGlobalScope;
const rpc = new Rpc<Worker<TestSuiteWorkerSpec>>(self);

const tests = {
  testCallRustFromWorker: async function () {
    const buffer = new SharedArrayBuffer(8);
    new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
    const uint8Part = new Uint8Array(buffer, 2, 4);
    const [result] = await self.callRust("array_multiply", [
      JSON.stringify(10),
      uint8Part,
    ]);
    expect(result.length, 4);
    expect(result[0], 30);
    expect(result[1], 40);
    expect(result[2], 50);
    expect(result[3], 60);
  },
  testCallRustNoReturnFromWorker: async function () {
    const buffer = new SharedArrayBuffer(8);
    new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
    const uint8Part = new Uint8Array(buffer, 2, 4);
    const result = await self.callRust("call_rust_no_return", [
      JSON.stringify(10),
      uint8Part,
    ]);
    expect(result.length, 0);
  },
  testCallRustInSameThreadSyncWithSignal: function () {
    const result = self.callRustInSameThreadSync("send_signal");
    expect(result.length, 0);
  },
  testCallRustFloat32ArrayFromWorker: async () => {
    // Using a normal array
    const input = new Float32Array([0.1, 0.9, 0.3]);
    const result = (
      await self.callRust("array_multiply", [JSON.stringify(10), input])
    )[0] as Float32Array;
    expect(result.length, 3);
    expect(result[0], 1);
    expect(result[1], 9);
    expect(result[2], 3);

    // Using a WrfArray
    const input2 = self.createBuffer(new Float32Array([0.1, 0.9, 0.3]));
    const result2 = (
      await self.callRust("array_multiply", [JSON.stringify(10), input2])
    )[0] as Float32Array;
    expect(result2.length, 3);
    expect(result2[0], 1);
    expect(result2[1], 9);
    expect(result2[2], 3);

    // Using a readonly WrfArray
    const input3 = self.createReadOnlyBuffer(new Float32Array([0.1, 0.9, 0.3]));

    const result3 = (
      await self.callRust("array_multiply", [JSON.stringify(10), input3])
    )[0] as Float32Array;
    expect(result3.length, 3);
    expect(result3[0], 1);
    expect(result3[1], 9);
    expect(result3[2], 3);
  },
  testCallRustInSameThreadSyncFloat32ArrayFromWorker: async () => {
    // Using a normal array
    const input = new Float32Array([0.1, 0.9, 0.3]);
    const result = self.callRustInSameThreadSync("array_multiply", [
      JSON.stringify(10),
      input,
    ])[0] as Float32Array;
    expect(result.length, 3);
    expect(result[0], 1);
    expect(result[1], 9);
    expect(result[2], 3);

    // Using a WrfArray
    const input2 = self.createBuffer(new Float32Array([0.1, 0.9, 0.3]));
    const result2 = self.callRustInSameThreadSync("array_multiply", [
      JSON.stringify(10),
      input2,
    ])[0] as Float32Array;
    expect(result2.length, 3);
    expect(result2[0], 1);
    expect(result2[1], 9);
    expect(result2[2], 3);

    // Using a readonly WrfArray
    const input3 = self.createReadOnlyBuffer(new Float32Array([0.1, 0.9, 0.3]));

    const result3 = self.callRustInSameThreadSync("array_multiply", [
      JSON.stringify(10),
      input3,
    ])[0] as Float32Array;
    expect(result3.length, 3);
    expect(result3[0], 1);
    expect(result3[1], 9);
    expect(result3[2], 3);
  },
};
export type TestSuiteTests = keyof typeof tests;

rpc.receive("initWasm", (port) => {
  self.initWrfUserWorkerRuntime(port);
});

rpc.receive("runTest", async (testName) => tests[testName]());

rpc.receive("sendWorker", function (array) {
  const data = self.deserializeWrfArrayFromPostMessage(array);
  console.log("got data", data);
});

rpc.receive("testSendWrfArrayToMainThread", function () {
  const buffer = new SharedArrayBuffer(8);
  new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
  const uint8Part = new Uint8Array(buffer, 2, 4);
  const wrfArray = self.callRustInSameThreadSync("array_multiply", [
    JSON.stringify(10),
    uint8Part,
  ])[0] as Uint8Array;

  return {
    array: self.serializeWrfArrayForPostMessage(wrfArray),
    subarray: self.serializeWrfArrayForPostMessage(wrfArray.subarray(1, 3)),
  };
});
rpc.receive("testCallRustInSameThreadSyncWithWrfbuffer", function () {
  const result = self.createBuffer(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]));
  const [result2] = self.callRustInSameThreadSync("array_multiply", [
    JSON.stringify(10),
    result,
  ]);

  return self.serializeWrfArrayForPostMessage(result2);
});
