// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

/// <reference lib="WebWorker" />

// The "Wrf WebWorker runtime" exposes some common Wrf functions inside your WebWorkers, like `callRust`.
//
// Include the output of this (wrf_web_worker_runtime.js) at the start of each worker, and initialize the runtime
// by calling `self.initWrfUserWorkerRuntime` with a `MessagePort` obtained by `wrfNewWorkerPort` (which is
// available on `window` in the main browser thread, and in any worker that already has the runtime running). You
// can pass the port to the worker using `postMessage`; just be sure to include it in the list of transferables.
//
// Currently this is only supported in WebAssembly, not when using CEF.

declare let self: WorkerGlobalScope;

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
  WrfParam,
  WrfParamType,
  WrfArray,
  RustWrfParam,
  MutableBufferData,
} from "./types";
import {
  getWrfBufferWasm,
  isWrfBuffer,
  overwriteTypedArraysWithWrfArrays,
  unregisterMutableBuffer,
  WrfBuffer,
  checkValidWrfArray,
} from "./wrf_buffer";
import { ZerdeParser } from "./zerde";

declare global {
  interface WorkerGlobalScope extends Worker {
    initWrfUserWorkerRuntime: (arg0: MessagePort) => void;
    wrfNewWorkerPort: () => MessagePort;
    callRust: CallRust;
    wrfInitialized: Promise<void> | undefined;
    callRustInSameThreadSync: CallRustInSameThreadSync;
    createBuffer: <T extends WrfArray>(data: T) => T;
    createReadOnlyBuffer: <T extends WrfArray>(data: T) => T;
    isWrfBuffer: (potentialWrfBuffer: unknown) => boolean;
    serializeWrfArrayForPostMessage: (wrfArray: any) => PostMessageTypedArray;
    deserializeWrfArrayFromPostMessage: (
      postMessageData: PostMessageTypedArray
    ) => Uint8Array;
  }
}

overwriteTypedArraysWithWrfArrays();

let _wrfWorkerRpc: Rpc<WebWorkerRpc>;

self.initWrfUserWorkerRuntime = (wrfWorkerPort: MessagePort) => {
  if (self.wrfInitialized) {
    throw new Error("Don't call initWrfUserWorkerRuntime twice");
  }

  self.wrfInitialized = new Promise((resolve) => {
    _wrfWorkerRpc = new Rpc(wrfWorkerPort);

    self.wrfNewWorkerPort = () => {
      const channel = new MessageChannel();
      _wrfWorkerRpc.send(
        MainWorkerChannelEvent.BindMainWorkerPort,
        channel.port1,
        [channel.port1]
      );
      return channel.port2;
    };

    _wrfWorkerRpc
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
          let wasmExports: WasmExports;
          function getExports() {
            return wasmExports;
          }

          const env = getWasmEnv({
            getExports,
            memory,
            taskWorkerSab,
            fileHandles: [], // TODO(JP): implement at some point..
            sendEventFromAnyThread: (eventPtr: BigInt) => {
              _wrfWorkerRpc.send(
                MainWorkerChannelEvent.SendEventFromAnyThread,
                eventPtr
              );
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
                memory,
                destructor,
                mutableDestructor,
                params
              );

            // TODO(JP): Allocate buffers on the wasm memory directly here.
            self.callRust = async (name, params = []): Promise<WrfParam[]> => {
              for (const param of params) {
                if (
                  typeof param !== "string" &&
                  !(param.buffer instanceof SharedArrayBuffer)
                ) {
                  console.warn(
                    "Consider passing Uint8Arrays backed by SharedArrayBuffer into `callRust` to prevent copying data"
                  );
                }
              }

              const result = await _wrfWorkerRpc.send(
                MainWorkerChannelEvent.CallRust,
                {
                  name,
                  params,
                }
              );
              return transformParamsFromRust(result);
            };

            // TODO(JP): Some of this code is duplicated with callRust/call_js; see if we can reuse some.
            self.callRustInSameThreadSync = (name, params = []): WrfParam[] => {
              const zerdeBuilder = makeZerdeBuilder(memory, wasmExports);
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
                      zerdeBuilder.sendU32(
                        param.buffer.__wrflibBufferData.bufferPtr
                      );
                      zerdeBuilder.sendU32(
                        param.buffer.__wrflibBufferData.bufferLen
                      );
                      zerdeBuilder.sendU32(
                        param.buffer.__wrflibBufferData.bufferCap
                      );
                    }
                  } else {
                    console.warn(
                      "Consider passing Uint8Arrays backed by WrfBuffer to prevent copying data"
                    );

                    const vecLen = param.byteLength;
                    const vecPtr = createWasmBuffer(memory, wasmExports, param);
                    zerdeBuilder.sendU32(getWrfParamType(param, false));
                    zerdeBuilder.sendU32(vecPtr);
                    zerdeBuilder.sendU32(vecLen);
                    zerdeBuilder.sendU32(vecLen);
                  }
                }
              }
              const returnPtr = wasmExports.callRustInSameThreadSync(
                appPtr,
                BigInt(zerdeBuilder.getData().byteOffset)
              );

              const zerdeParser = new ZerdeParser(memory, Number(returnPtr));
              const returnParams = zerdeParser.parseWrfParams();
              return transformParamsFromRust(returnParams);
            };

            self.createBuffer = (data) => {
              const bufferLen = data.byteLength;
              const bufferPtr = createWasmBuffer(memory, wasmExports, data);
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

            self.createReadOnlyBuffer = (data) => {
              const bufferPtr = createWasmBuffer(memory, wasmExports, data);
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

            self.isWrfBuffer = isWrfBuffer;

            // TODO(JP): Somewhat duplicated with the other implementation.
            self.serializeWrfArrayForPostMessage = (
              wrfArray: any
            ): PostMessageTypedArray => {
              if (
                !(
                  typeof wrfArray === "object" &&
                  self.isWrfBuffer(wrfArray.buffer)
                )
              ) {
                throw new Error(
                  "Only pass Wrf arrays to serializeWrfArrayForPostMessage"
                );
              }
              const wrfBuffer = wrfArray.buffer;
              if (wrfBuffer.readonly) {
                wasmExports.incrementArc(
                  BigInt(wrfBuffer.__wrflibBufferData.arcPtr)
                );
              } else {
                unregisterMutableBuffer(wrfBuffer);
              }
              return {
                bufferData: wrfBuffer.__wrflibBufferData,
                byteOffset: wrfArray.byteOffset,
                byteLength: wrfArray.byteLength,
              };
            };

            self.deserializeWrfArrayFromPostMessage = (
              postMessageData: PostMessageTypedArray
            ): Uint8Array => {
              const wrfBuffer = getWrfBufferWasm(
                memory,
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

            resolve();
          });
        }
      );
  });
};
