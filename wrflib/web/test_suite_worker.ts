// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import "./wrf_user_worker_runtime";
import { expect } from "./wrf_test";
import { Rpc } from "./common";
import { PostMessageTypedArray } from "./types";

declare let self: WorkerGlobalScope;
const rpc = new Rpc(self);

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
};
export type TestSuiteTests = keyof typeof tests;

rpc.receive<MessagePort, void>("init_wasm", (port) => {
  self.initWrfUserWorkerRuntime(port);
});

rpc.receive<TestSuiteTests, void>("run_test", (testName) => {
  tests[testName]();
});

rpc.receive<PostMessageTypedArray, void>("send_worker", function (array) {
  const data = self.deserializeWrfArrayFromPostMessage(array);
  console.log("got data", data);
});

rpc.receive("test_send_wrf_array_to_main_thread", function () {
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
rpc.receive("test_call_rust_in_same_thread_sync_with_wrfbuffer", function () {
  const [result] = self.callRustInSameThreadSync("make_wrfbuffer");
  const [result2] = self.callRustInSameThreadSync("array_multiply", [
    JSON.stringify(10),
    result,
  ]);

  return self.serializeWrfArrayForPostMessage(result2);
});
