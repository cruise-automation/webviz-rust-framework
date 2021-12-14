// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// This file should only be imported by WebWorkers
/// <reference lib="WebWorker" />

import { FileHandle, WasmEnv, WasmExports } from "./types";
import { ZerdeBuilder } from "./zerde";

////////////////////////////////////////////////////////////////
// RPC
////////////////////////////////////////////////////////////////

// Taken from https://github.com/cruise-automation/webviz/blob/6a4226bc2959444704d650d8c55cea4f4220c75c/packages/webviz-core/src/util/Rpc.js
// TODO(JP): Maybe release as a package?
// TODO(JP): Also be sure to include the tests at some point: https://github.com/cruise-automation/webviz/blob/6a4226bc2959444704d650d8c55cea4f4220c75c/packages/webviz-core/src/util/Rpc.test.js

// this type mirrors the MessageChannel and MessagePort APIs which are available on
// instances of web-workers and shared-workers respectively, as well as avaiable on
// 'global' within them.
export interface Channel {
  postMessage(data: unknown, transfer?: unknown[]): void;
  onmessage: null | ((ev: MessageEvent) => unknown);
}

const RESPONSE = "$$RESPONSE";
const ERROR = "$$ERROR";

// helper function to create linked channels for testing
function _createLinkedChannels(): { local: Channel; remote: Channel } {
  const local: Channel = {
    onmessage,

    postMessage(data: unknown, _transfer?: Array<ArrayBuffer>) {
      const ev = new MessageEvent("message", { data });
      if (remote.onmessage) {
        remote.onmessage(ev);
      }
    },
  };

  const remote: Channel = {
    onmessage,

    postMessage(data, _transfer) {
      const ev = new MessageEvent("message", { data });
      if (local.onmessage) {
        local.onmessage(ev);
      }
    },
  };
  return { local, remote };
}

// This class allows you to hook up bi-directional async calls across web-worker
// boundaries where a single call to or from a worker can 'wait' on the response.
// Errors in receivers are propigated back to the caller as a rejection.
// It also supports returning transferrables over the web-worker postMessage api,
// which was the main shortcomming with the worker-rpc npm module.
// To attach rpc to an instance of a worker in the main thread:
//   const rpc = new Rpc(workerInstace);
// To attach rpc within an a web worker:
//   const rpc = new Rpc(global);
// Check out the tests for more examples.
export class Rpc {
  static transferrables = "$$TRANSFERRABLES";
  _channel = undefined;
  _messageId = 0;
  _pendingCallbacks: Record<number, (arg0: any) => void> = {};
  _receivers = new Map();

  constructor(channel: Channel) {
    this._channel = channel;
    if (this._channel.onmessage) {
      throw new Error(
        "channel.onmessage is already set. Can only use one Rpc instance per channel."
      );
    }
    this._channel.onmessage = this._onChannelMessage;
  }

  _onChannelMessage = (ev: MessageEvent): void => {
    const { id, topic, data } = ev.data;
    if (topic === RESPONSE) {
      this._pendingCallbacks[id](ev.data);
      delete this._pendingCallbacks[id];
      return;
    }
    // invoke the receive handler in a promise so if it throws synchronously we can reject
    new Promise((resolve) => {
      const handler = this._receivers.get(topic);
      if (!handler) {
        throw new Error(`no receiver registered for ${topic}`);
      }
      // This works both when `handler` returns a value or a Promise.
      resolve(handler(data));
    })
      .then((result) => {
        if (!result) {
          this._channel.postMessage({ topic: RESPONSE, id });
          return;
        }
        const transferrables = result[Rpc.transferrables];
        delete result[Rpc.transferrables];
        const message = {
          topic: RESPONSE,
          id,
          data: result,
        };
        this._channel.postMessage(message, transferrables);
      })
      .catch((err) => {
        const message = {
          topic: RESPONSE,
          id,
          data: {
            [ERROR]: true,
            name: err.name,
            message: err.message,
            stack: err.stack,
          },
        };
        this._channel.postMessage(message);
      });
  };

  // send a message across the rpc boundary to a receiver on the other side
  // this returns a promise for the receiver's response.  If there is no registered
  // receiver for the given topic, this method throws
  send<TResult>(
    topic: string,
    data?: unknown,
    transfer?: any[]
  ): Promise<TResult> {
    const id = this._messageId++;
    const message = { topic, id, data };
    const result = new Promise<any>((resolve, reject) => {
      this._pendingCallbacks[id] = (info: any) => {
        if (info.data && info.data[ERROR]) {
          const error = new Error(info.data.message);
          error.name = info.data.name;
          error.stack = info.data.stack;
          reject(error);
        } else {
          resolve(info.data);
        }
      };
    });
    this._channel.postMessage(message, transfer);
    return result;
  }

  // register a receiver for a given message on a topic
  // only one receiver can be registered per topic and currently
  // 'deregistering' a receiver is not supported since this is not common
  receive<T, TOut>(topic: string, handler: (arg0: T) => TOut): void {
    if (this._receivers.has(topic)) {
      throw new Error(`Receiver already registered for topic: ${topic}`);
    }
    this._receivers.set(topic, handler);
  }
}

////////////////////////////////////////////////////////////////
// Mutex
////////////////////////////////////////////////////////////////

const MUTEX_UNLOCKED = 0;
const MUTEX_LOCKED = 1;

export const mutexLock = (sabi32: Int32Array, offset: number): void => {
  // This needs to be in a loop, because between the `wait` and `compareExchange` another thread might
  // take the Mutex.
  // eslint-disable-next-line no-constant-condition
  while (true) {
    if (
      Atomics.compareExchange(sabi32, offset, MUTEX_UNLOCKED, MUTEX_LOCKED) ==
      MUTEX_UNLOCKED
    ) {
      return;
    }
    Atomics.wait(sabi32, offset, MUTEX_LOCKED);
  }
};

export const mutexUnlock = (sabi32: Int32Array, offset: number): void => {
  if (
    Atomics.compareExchange(sabi32, offset, MUTEX_LOCKED, MUTEX_UNLOCKED) !=
    MUTEX_LOCKED
  ) {
    throw new Error("Called mutex_unlock on an already unlocked mutex");
  }
  Atomics.notify(sabi32, offset, 1);
};

////////////////////////////////////////////////////////////////
// Task worker
////////////////////////////////////////////////////////////////

export const TW_SAB_MUTEX_PTR = 0;
export const TW_SAB_MESSAGE_COUNT_PTR = 1;

// Initialize a SharedArrayBuffer used to communicate with task_worker.ts. This
// is a one-way communication channel; use pointers into `memory` for communicating
// information back.
//
// We use this because we typically can't use `postMessage`; see task_worker.ts
// for more details.
//
// Format:
// * i32 (4 bytes)         - read/write mutex
// * i32 (4 bytes)         - number of messages in queue (notify on this to wake up the task worker - it will
//                           read this before taking a mutex, but then reread it after taking the mutex)
// * n * u32 (n * 4 bytes) - pointers to messages serialized with `ZerdeBuilder`
export const initTaskWorkerSab = (): SharedArrayBuffer => {
  const bufferSizeBytes = 10000;
  const taskWorkerSab = new SharedArrayBuffer(bufferSizeBytes);
  const taskWorkerSabi32 = new Int32Array(taskWorkerSab);
  taskWorkerSabi32[TW_SAB_MUTEX_PTR] = MUTEX_UNLOCKED;
  taskWorkerSabi32[TW_SAB_MESSAGE_COUNT_PTR] = 0;
  return taskWorkerSab;
};

// Append a new message pointer to the SharedArrayBuffer used by task_worker.ts,
// and wake it up so it can process this new message (unless it's currently in polling
// mode, in that case the `Atomics.notify` will just not do anything).
const sendTaskWorkerMessage = (
  taskWorkerSab: SharedArrayBuffer,
  twMessagePtr: number
) => {
  const taskWorkerSabi32 = new Int32Array(taskWorkerSab);
  mutexLock(taskWorkerSabi32, TW_SAB_MUTEX_PTR);

  const currentNumberOfMessages = taskWorkerSabi32[TW_SAB_MESSAGE_COUNT_PTR];
  // Use unsigned numbers for the actual pointer, since they can be >2GB.
  new Uint32Array(taskWorkerSab)[currentNumberOfMessages + 2] = twMessagePtr;
  taskWorkerSabi32[TW_SAB_MESSAGE_COUNT_PTR] = currentNumberOfMessages + 1;

  mutexUnlock(taskWorkerSabi32, TW_SAB_MUTEX_PTR);
  Atomics.notify(taskWorkerSabi32, 1);
};

////////////////////////////////////////////////////////////////
// Common wasm functions
////////////////////////////////////////////////////////////////

export const copyUint8ArrayToRustBuffer = (
  inputBuffer: Uint8Array,
  outputBuffer: ArrayBuffer,
  outputPtr: number
): void => {
  const u8len = inputBuffer.byteLength;
  const u8out = new Uint8Array(outputBuffer, outputPtr, u8len);
  u8out.set(inputBuffer);
};

export const makeZerdeBuilder = (
  memory: WebAssembly.Memory,
  wasmExports: WasmExports
): ZerdeBuilder => {
  const slots = 1024;
  // We have get memory.buffer *after* calling `allocWasmMessage`, because
  // there's a good chance it'll get swapped out (if it needed to grow the buffer).
  const byteOffset = Number(wasmExports.allocWasmMessage(BigInt(slots * 4)));
  return new ZerdeBuilder({
    buffer: memory.buffer,
    byteOffset: byteOffset,
    slots,
    growCallback: (_buffer, oldByteOffset, newBytes) => {
      const newByteOffset = Number(
        wasmExports.reallocWasmMessage(BigInt(oldByteOffset), BigInt(newBytes))
      );
      // We have get memory.buffer *after* calling `reallocWasmMessage`, because
      // there's a good chance it'll get swapped out (if it needed to grow the buffer).
      return { buffer: memory.buffer, byteOffset: newByteOffset };
    },
  });
};

export const getWasmEnv = ({
  getExports,
  memory,
  taskWorkerSab,
  fileHandles,
  sendEventFromAnyThread,
  threadSpawn,
  baseUri,
}: {
  getExports: () => WasmExports;
  memory: WebAssembly.Memory;
  taskWorkerSab: SharedArrayBuffer;
  fileHandles: FileHandle[];
  sendEventFromAnyThread: (eventPtr: BigInt) => void;
  threadSpawn: (ctxPtr: BigInt) => void;
  baseUri;
}): WasmEnv => {
  const fileReaderSync = new FileReaderSync();

  const parseString = (ptr: number, len: number) => {
    let out = "";
    // Can't use TextDecoder here since it doesn't work with SharedArrayBuffer.
    // TODO(JP): If it becomes important enough, we can see if making a copy to a regular
    // ArrayBuffer and then using TextDecoder is faster than what we do here.
    const array = new Uint32Array(memory.buffer, ptr, len);
    for (let i = 0; i < len; i++) {
      out += String.fromCharCode(array[i]);
    }
    return out;
  };

  return {
    memory,
    _consoleLog: (charsPtr, len, error) => {
      const out = parseString(parseInt(charsPtr), parseInt(len));
      if (error) {
        console.error(out);
      } else {
        console.log(out);
      }
    },
    readUserFileRange: (userFileId, bufPtr, bufLen, fileOffset) => {
      const file = fileHandles[userFileId];
      const start = Number(fileOffset);
      const end = start + Number(bufLen);
      if (file.lastReadStart <= start && start < file.lastReadEnd) {
        console.warn(
          `Read start (${start}) fell in the range of the last read (${file.lastReadStart}-${file.lastReadEnd}); ` +
            "this usually happens if you don't use BufReader or if you don't use BufReader.seek_relative."
        );
      }
      file.lastReadStart = start;
      file.lastReadEnd = end;
      // TODO(JP): This creates a new buffer instead of reading directly into the wasm memory.
      // Maybe we can avoid this by using a stream with a ReadableStreamBYOBReader, but that is
      // asynchronous, so we'd have to do a dance with another thread and atomics and all that,
      // and I don't know if that overhead would be worth it..
      const buffer = fileReaderSync.readAsArrayBuffer(
        file.file.slice(start, end)
      );
      copyUint8ArrayToRustBuffer(
        new Uint8Array(buffer),
        memory.buffer,
        Number(bufPtr)
      );
      return BigInt(buffer.byteLength);
    },
    performanceNow: () => {
      return performance.now();
    },
    threadSpawn: (ctxPtr) => {
      threadSpawn(ctxPtr);
    },
    _sendEventFromAnyThread: (eventPtr: BigInt) => {
      sendEventFromAnyThread(eventPtr);
    },
    readUrlSync: (urlPtr, urlLen, bufPtrOut, bufLenOut) => {
      const url = parseString(urlPtr, urlLen);
      const request = new XMLHttpRequest();
      request.responseType = "arraybuffer";
      request.open("GET", new URL(url, baseUri).href, false /* synchronous */);
      request.send(null);

      if (request.status === 200) {
        const exports = getExports();
        const response = request.response as ArrayBuffer;
        const outputBufPtr = Number(
          exports.allocWasmVec(BigInt(response.byteLength))
        );
        copyUint8ArrayToRustBuffer(
          new Uint8Array(request.response),
          memory.buffer,
          outputBufPtr
        );
        new Uint32Array(memory.buffer, bufPtrOut, 1)[0] = outputBufPtr;
        new Uint32Array(memory.buffer, bufLenOut, 1)[0] =
          request.response.byteLength;
        return 1;
      } else {
        return 0;
      }
    },
    randomU64: () =>
      new BigUint64Array(
        self.crypto.getRandomValues(new Uint32Array(2)).buffer
      )[0],
    sendTaskWorkerMessage: (twMessagePtr) => {
      sendTaskWorkerMessage(taskWorkerSab, parseInt(twMessagePtr));
    },
  };
};
