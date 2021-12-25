// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import {
  Rpc,
  getWasmEnv,
  initThreadLocalStorageAndStackOtherWorkers,
} from "./common";
import { AsyncWorkerEvent, AsyncWorkerRunValue, WasmExports } from "./types";

const rpc = new Rpc(self);

rpc.receive<AsyncWorkerRunValue, void>(
  AsyncWorkerEvent.Run,
  ({
    wasmModule,
    memory,
    taskWorkerSab,
    ctxPtr,
    fileHandles,
    baseUri,
    tlsAndStackData,
  }) => {
    const sendEventFromAnyThread = (eventPtr: BigInt) => {
      rpc.send(AsyncWorkerEvent.SendEventFromAnyThread, { eventPtr });
    };
    const threadSpawn = (ctxPtr: BigInt) => {
      rpc.send(AsyncWorkerEvent.ThreadSpawn, { ctxPtr });
    };

    let exports: WasmExports;
    const getExports = () => {
      return exports;
    };
    const env = getWasmEnv({
      getExports,
      memory,
      taskWorkerSab,
      fileHandles,
      sendEventFromAnyThread,
      threadSpawn,
      baseUri,
    });

    return new Promise<void>((resolve, reject) => {
      WebAssembly.instantiate(wasmModule, { env }).then((instance) => {
        exports = instance.exports;
        initThreadLocalStorageAndStackOtherWorkers(exports, tlsAndStackData);
        // TODO(Paras): Eventually call `processWasmEvents` instead of a custom exported function.
        exports.runFunctionPointer(ctxPtr);
        resolve();
      }, reject);
    });
  }
);
