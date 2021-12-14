// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import { Rpc } from "./common";
import { TestSuiteTests } from "./test_suite_worker";
import { PostMessageTypedArray } from "./types";
import { wrfBufferTests } from "./wrf_buffer.test";
import * as wrf from "./wrf_runtime";
import {
  expect,
  expectDeallocation as _expectDeallocation,
  setInTest,
} from "./wrf_test";

const expectDeallocation = (buffer: Uint8Array) =>
  _expectDeallocation(wrf.callRust, buffer);

const rpc = new Rpc(
  new Worker(new URL("./test_suite_worker.ts", import.meta.url))
);

const runWorkerTest = (testName: TestSuiteTests) => async () => {
  return await rpc.send("run_test", testName);
};

const filename = "target/wasm32-unknown-unknown/release/test_suite-xform.wasm";
wrf.initialize({ filename }).then(() => {
  // Initialize the worker by sending a "wrf worker port" to it in the first message.
  if (wrf.jsRuntime === "wasm") {
    const wrfWorkerPort = wrf.wrfNewWorkerPort();
    rpc.send("init_wasm", wrfWorkerPort, [wrfWorkerPort]);
  }

  wrf.registerCallJsCallbacks({
    log(params) {
      console.log("log fn called", params[0]);
      const div = document.createElement("div");
      div.innerText = "log fn called: " + params[0];
      document.getElementById("js_root").append(div);
    },
    sendWorker(params) {
      const toSend = params[0] as Uint8Array;
      console.log("sending data", toSend);
      // Note: uncomment to see the error about sending typed arrays
      // worker.postMessage(buffers[0]);
      rpc.send("send_worker", wrf.serializeWrfArrayForPostMessage(toSend));
    },
  });

  const runtimeSpecificTests =
    wrf.jsRuntime === "wasm"
      ? {
          "Call rust from worker": runWorkerTest("testCallRustFromWorker"),
          "Call rust (no return) from worker": runWorkerTest(
            "testCallRustNoReturnFromWorker"
          ),
          "Send wrf array to main thread": async () => {
            const result = await rpc.send<{
              array: PostMessageTypedArray;
              subarray: PostMessageTypedArray;
            }>("test_send_wrf_array_to_main_thread");

            const array = wrf.deserializeWrfArrayFromPostMessage(result.array);
            const subarray = wrf.deserializeWrfArrayFromPostMessage(
              result.subarray
            );

            expect(array.length, 4);
            expect(array[0], 30);
            expect(array[1], 40);
            expect(array[2], 50);
            expect(array[3], 60);

            expect(subarray.length, 2);
            expect(subarray[0], 40);
            expect(subarray[1], 50);
          },
          "Call Rust in same thread with wrfbuffer": async () => {
            const result = await rpc.send<PostMessageTypedArray>(
              "test_call_rust_in_same_thread_sync_with_wrfbuffer"
            );
            const array = wrf.deserializeWrfArrayFromPostMessage(result);
            expect(array.length, 8);
            expect(array[0], 10);
            expect(array[1], 20);
            expect(array[2], 30);
            expect(array[3], 40);
            expect(array[4], 50);
            expect(array[5], 60);
            expect(array[6], 70);
            expect(array[7], 80);
          },
          "Send signal from worker": runWorkerTest(
            "testCallRustInSameThreadSyncWithSignal"
          ),
        }
      : {
          "Call Rust (in same thread)": () => {
            const buffer = new SharedArrayBuffer(8);
            new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
            const uint8Part = new Uint8Array(buffer, 2, 4);
            const [result] = wrf.callRustInSameThreadSync("array_multiply", [
              JSON.stringify(10),
              uint8Part,
            ]);
            expect(result.length, 4);
            expect(result[0], 30);
            expect(result[1], 40);
            expect(result[2], 50);
            expect(result[3], 60);
          },
        };

  const tests = {
    "Call Rust": async () => {
      const buffer = new SharedArrayBuffer(8);
      new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
      const uint8Part = new Uint8Array(buffer, 2, 4);
      const [result] = await wrf.callRust("array_multiply", [
        JSON.stringify(10),
        uint8Part,
      ]);
      expect(result.length, 4);
      expect(result[0], 30);
      expect(result[1], 40);
      expect(result[2], 50);
      expect(result[3], 60);
    },
    "Call Rust (no return)": async () => {
      const buffer = new SharedArrayBuffer(8);
      new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
      const uint8Part = new Uint8Array(buffer, 2, 4);
      const result = await wrf.callRust("call_rust_no_return", [
        JSON.stringify(10),
        uint8Part,
      ]);
      expect(result.length, 0);
    },
    "Call Rust (string return)": async () => {
      const buffer = new SharedArrayBuffer(8);
      const data = new Uint8Array(buffer);
      data.set([1, 2, 3, 4, 5, 6, 7, 8]);
      const [result] = await wrf.callRust("total_sum", [data]);
      expect(result, "36");
    },
    "Call Rust (with WrfBuffer)": async () => {
      const buffer = (await wrf.callRust("make_wrfbuffer"))[0] as Uint8Array;
      const result = (
        await wrf.callRust("array_multiply", [JSON.stringify(10), buffer])
      )[0] as Uint8Array;
      expect(result.length, 8);
      expect(result[0], 10);
      expect(result[1], 20);
      expect(result[2], 30);
      expect(result[3], 40);
      expect(result[4], 50);
      expect(result[5], 60);
      expect(result[6], 70);
      expect(result[7], 80);
      return Promise.all([
        expectDeallocation(buffer),
        expectDeallocation(result),
      ]);
    },
    "Call Rust (with Mutable WrfBuffer)": async () => {
      // TODO(Paras): Add enforcement of readonly WrfArrays and test it.
      // const [buffer] = await wrf.callRust("make_wrfbuffer");
      // let err;
      // try {
      //     buffer[0] = 0;
      // } catch (e) {
      //     err = e;
      // } finally {
      //     expect(err?.message, "Cannot mutate a read-only array");
      // }

      const mutableBuffer = (
        await wrf.callRust("make_mutable_wrfbuffer")
      )[0] as Uint8Array;
      expect(mutableBuffer.length, 8);
      expect(mutableBuffer[0], 1);
      expect(mutableBuffer[1], 2);
      expect(mutableBuffer[2], 3);
      expect(mutableBuffer[3], 4);
      expect(mutableBuffer[4], 5);
      expect(mutableBuffer[5], 6);
      expect(mutableBuffer[6], 7);
      expect(mutableBuffer[7], 8);

      // Mutate the buffer to ensure the changes are detected in Rust code
      mutableBuffer[0] = 0;
      mutableBuffer[1] = 0;
      mutableBuffer[2] = 0;
      mutableBuffer[3] = 0;

      const result = (
        await wrf.callRust("array_multiply", [
          JSON.stringify(10),
          mutableBuffer,
        ])
      )[0] as Uint8Array;
      expect(result.length, 8);
      expect(result[0], 0);
      expect(result[1], 0);
      expect(result[2], 0);
      expect(result[3], 0);
      expect(result[4], 50);
      expect(result[5], 60);
      expect(result[6], 70);
      expect(result[7], 80);

      return Promise.all([
        expectDeallocation(mutableBuffer),
        expectDeallocation(result),
      ]);
    },
    ...runtimeSpecificTests,
    ...wrfBufferTests,
  };

  const makeButtons = () => {
    const jsRoot = document.getElementById("js_root");

    const runAllButton = document.createElement("button");
    runAllButton.innerText = "Run All Tests";
    runAllButton.onclick = async () => {
      setInTest(true);
      for (const [testName, test] of Object.entries(tests)) {
        console.log(`Running test: ${testName}`);
        await test();
      }
      setInTest(false);
    };
    const buttonDiv = document.createElement("div");
    buttonDiv.append(runAllButton);
    jsRoot.append(buttonDiv);

    for (const [name, test] of Object.entries(tests)) {
      const button = document.createElement("button");
      button.innerText = name;
      button.onclick = async () => {
        setInTest(true);
        await test();
        setInTest(false);
      };

      const buttonDiv = document.createElement("div");
      buttonDiv.append(button);
      jsRoot.append(buttonDiv);
    }
  };

  if (document.readyState !== "loading") {
    makeButtons();
  } else {
    document.addEventListener("DOMContentLoaded", makeButtons);
  }
});
