// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

/// <reference lib="WebWorker" />

// The "Wrf user worker runtime" exposes some common Wrf functions inside your WebWorkers, like `callRust`.
//
// Include the output of this (wrf_user_worker_runtime.js) at the start of each worker, and initialize the runtime
// by calling `self.initWrfUserWorkerRuntime` with a `MessagePort` obtained by `wrfNewWorkerPort` (which is
// available on `window` in the main browser thread, and in any worker that already has the runtime running). You
// can pass the port to the worker using `postMessage`; just be sure to include it in the list of transferables.
//
// Currently this is only supported in WebAssembly, not when using CEF.

declare let self: WorkerGlobalScope;

import {
  copyUint8ArrayToRustBuffer,
  getWasmEnv,
  makeZerdeBuilder,
  Rpc,
} from "./common";
import {
  BufferData,
  CallRust,
  CallRustInSameThreadSync,
  PostMessageTypedArray,
  UserWorkerEvent,
  UserWorkerInitReturnValue,
  WasmExports,
  WrfParam,
  WrfParamType,
} from "./types";
import {
  getCachedUint8Buffer,
  getWrfBufferWasm,
  isWrfBuffer,
  overwriteTypedArraysWithWrfArrays,
  unregisterMutableBuffer,
  WrfBuffer,
  wrfArrayCoversWrfBuffer,
} from "./wrf_buffer";
import { ZerdeParser } from "./zerde";

declare global {
  interface WorkerGlobalScope extends Worker {
    initWrfUserWorkerRuntime: (arg0: MessagePort) => void;
    wrfNewWorkerPort: () => MessagePort;
    callRust: CallRust;
    wrfInitialized: Promise<void>;
    callRustInSameThreadSync: CallRustInSameThreadSync;
    isWrfBuffer: (potentialWrfBuffer: unknown) => boolean;
    serializeWrfArrayForPostMessage: (wrfArray: any) => PostMessageTypedArray;
    deserializeWrfArrayFromPostMessage: (
      postMessageData: PostMessageTypedArray
    ) => Uint8Array;
  }
}

overwriteTypedArraysWithWrfArrays();

let _wrfWorkerRpc: Rpc;

self.initWrfUserWorkerRuntime = (wrfWorkerPort: MessagePort) => {
  if (self.wrfInitialized) {
    throw new Error("Don't call initWrfUserWorkerRuntime twice");
  }

  self.wrfInitialized = new Promise((resolve) => {
    _wrfWorkerRpc = new Rpc(wrfWorkerPort);

    self.wrfNewWorkerPort = () => {
      const channel = new MessageChannel();
      _wrfWorkerRpc.send(
        UserWorkerEvent.BindUserWorkerPortOnMainThread,
        { port: channel.port1 },
        [channel.port1]
      );
      return channel.port2;
    };

    _wrfWorkerRpc
      .send<UserWorkerInitReturnValue>(UserWorkerEvent.Init)
      .then(({ wasmModule, memory, taskWorkerSab, baseUri, appPtr }) => {
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
              UserWorkerEvent.SendEventFromAnyThread,
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

          const deconstructor = (arcPtr: number) => {
            wasmExports.decrementArc(BigInt(arcPtr));
          };

          const mutableDeconstructor = ({
            bufferPtr,
            bufferLen,
            bufferCap,
          }: BufferData) => {
            wasmExports.deallocVec(
              BigInt(bufferPtr),
              BigInt(bufferLen),
              BigInt(bufferCap)
            );
          };

          function transformParamsFromRust(params: (string | BufferData)[]) {
            return params.map((param) => {
              if (typeof param === "string") {
                return param;
              } else {
                const wrfBuffer = getWrfBufferWasm(
                  memory,
                  param,
                  deconstructor,
                  mutableDeconstructor
                );
                return getCachedUint8Buffer(
                  wrfBuffer,
                  // This actually creates a WrfUint8Array as this was overwritten above in overwriteTypedArraysWithWrfArrays()
                  new Uint8Array(wrfBuffer, param.bufferPtr, param.bufferLen)
                );
              }
            });
          }

          // TODO(JP): Allocate buffers on the wasm memory directly here.
          self.callRust = async (name, params): Promise<WrfParam[]> => {
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

            const result = await _wrfWorkerRpc.send<(string | BufferData)[]>(
              UserWorkerEvent.CallRust,
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
                  if (!wrfArrayCoversWrfBuffer(param)) {
                    throw new Error(
                      "callRustInSameThreadSync only supports buffers that span the entire underlying WrfBuffer"
                    );
                  }
                  if (param.buffer.readonly) {
                    zerdeBuilder.sendU32(WrfParamType.ReadOnlyBuffer);

                    // WrfParam parsing code will construct an Arc without incrementing
                    // the count, so we do it here ahead of time.
                    wasmExports.incrementArc(
                      BigInt(param.buffer.__wrflibBufferData.arcPtr)
                    );
                    zerdeBuilder.sendU32(
                      param.buffer.__wrflibBufferData.arcPtr
                    );
                  } else {
                    // TODO(Paras): User should not be able to access the buffer after
                    // passing it to Rust here
                    unregisterMutableBuffer(param.buffer);
                    zerdeBuilder.sendU32(WrfParamType.Buffer);
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
                  const vecPtr = Number(
                    wasmExports.allocWasmVec(BigInt(vecLen))
                  );
                  copyUint8ArrayToRustBuffer(param, memory.buffer, vecPtr);
                  zerdeBuilder.sendU32(WrfParamType.Buffer);
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
              deconstructor,
              mutableDeconstructor
            );
            return new Uint8Array(
              wrfBuffer,
              postMessageData.byteOffset,
              postMessageData.byteLength
            );
          };

          resolve();
        });
      });
  });
};
