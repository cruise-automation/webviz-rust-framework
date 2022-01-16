// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import { cursorMap } from "./cursor_map";
import { copyArrayToRustBuffer, getWrfParamType } from "./common";
import { makeTextarea, TextareaEvent } from "./make_textarea";
import {
  CallRust,
  CallJsCallback,
  CallRustInSameThreadSync,
  WrfParam,
  PostMessageTypedArray,
  CreateBuffer,
  WrfParamType,
} from "./types";
import {
  getCachedWrfBuffer,
  overwriteTypedArraysWithWrfArrays,
  isWrfBuffer,
  checkValidWrfArray,
  getWrfBufferCef,
  WrfBuffer,
} from "./wrf_buffer";
import { ZerdeBuilder } from "./zerde";
import { zerdeKeyboardHandlers } from "./zerde_keyboard_handlers";
import { WorkerEvent } from "./rpc_types";

type CefParams = (string | [ArrayBuffer, WrfParamType])[];
type CefBufferData = [ArrayBuffer, number | undefined, WrfParamType];
type FromCefParams = (string | CefBufferData)[];
declare global {
  interface Window {
    // Defined externally in `cef_browser.rs`.
    cefCallRust: (name: string, params: CefParams, callbackId: number) => void;
    cefCallRustInSameThreadSync: (
      name: string,
      params: CefParams
    ) => FromCefParams;
    cefReadyForMessages: () => void;
    cefCreateArrayBuffer: (
      size: number,
      paramType: WrfParamType
    ) => CefBufferData;
    cefHandleKeyboardEvent: (buffer: ArrayBuffer) => void;
    cefTriggerCut: () => void;
    cefTriggerCopy: () => void;
    cefTriggerPaste: () => void;
    cefTriggerSelectAll: () => void;

    fromCefSetMouseCursor: (cursor: number) => void;
    fromCefSetIMEPosition: (x: number, y: number) => void;
    fromCefCallJsFunction: (name: string, params: FromCefParams) => void;
  }
}

let newCallbackId = 0;
// keeping track of pending callbacks from rust side
const pendingCallbacks: Record<number, (arg0: WrfParam[]) => void> = {};

export const callRust: CallRust = (name, params = []) => {
  const cefParams: CefParams = params.map((param) => {
    if (typeof param === "string") {
      return param;
    } else {
      if (isWrfBuffer(param.buffer)) {
        checkValidWrfArray(param);
        const wrfBuffer = param.buffer as WrfBuffer;
        return [
          wrfBuffer.__wrflibWasmBuffer,
          getWrfParamType(param, wrfBuffer.readonly),
        ];
      }
      const paramType = getWrfParamType(param, false);
      const [cefBuffer] = window.cefCreateArrayBuffer(param.length, paramType);
      copyArrayToRustBuffer(param, cefBuffer, 0);
      return [cefBuffer, paramType];
    }
  });
  const callbackId = newCallbackId++;
  const promise = new Promise<WrfParam[]>((resolve, _reject) => {
    pendingCallbacks[callbackId] = (data) => {
      // TODO(Dmitry): implement retrun_error on rust side and use reject(...) to communicate the error
      resolve(data);
    };
  });
  window.cefCallRust(name, cefParams, callbackId);
  return promise;
};

function _wrflibReturnParams(params: WrfParam[]) {
  const callbackId = JSON.parse(params[0] as string);
  pendingCallbacks[callbackId](params.slice(1));
  delete pendingCallbacks[callbackId];
}

// Initial set of framework-specific functions
const fromCefJsFunctions: Record<string, CallJsCallback> = {
  _wrflibReturnParams,
};

/// Users must call this function to register functions as runnable from
/// Rust via `[Cx::call_js]`.
export const registerCallJsCallbacks = (
  fns: Record<string, CallJsCallback>
): void => {
  // Check that all new functions are unique
  for (const key of Object.keys(fns)) {
    if (key in fromCefJsFunctions) {
      throw new Error(`Error: overwriting existing function "${key}"`);
    }
  }

  Object.assign(fromCefJsFunctions, fns);
  window.cefReadyForMessages();
};

/// Users must call this function to unregister functions as runnable from
/// Rust via `[Cx::call_js]`.
export const unregisterCallJsCallbacks = (fnNames: string[]): void => {
  fnNames.forEach((name) => {
    // Check that functions are registered
    if (!(name in fromCefJsFunctions)) {
      throw new Error(`Error: unregistering non-existent function "${name}"`);
    }

    delete fromCefJsFunctions[name];
  });
};

const transformReturnParams = (returnParams: FromCefParams) =>
  returnParams.map((param) => {
    if (typeof param === "string") {
      return param;
    } else {
      const [buffer, arcPtr, paramType] = param;
      const wrfBuffer = getWrfBufferCef(buffer, arcPtr, paramType);

      if (paramType === WrfParamType.String) {
        throw new Error("WrfParam buffer type called with string paramType");
      }

      // These are actually WrfArray types, since we overwrite TypedArrays in overwriteTypedArraysWithWrfArrays()
      const ParamTypeToArrayConstructor = {
        [WrfParamType.U8Buffer]: Uint8Array,
        [WrfParamType.ReadOnlyU8Buffer]: Uint8Array,
        [WrfParamType.F32Buffer]: Float32Array,
        [WrfParamType.ReadOnlyF32Buffer]: Float32Array,
      };

      // Creating array with stable identity as that's what underlying underlying API expects
      return getCachedWrfBuffer(
        wrfBuffer,
        new ParamTypeToArrayConstructor[paramType](wrfBuffer)
      );
    }
  });

// TODO(JP): Some of this code is duplicated with callRust/call_js; see if we can reuse some.
export const callRustInSameThreadSync: CallRustInSameThreadSync = (
  name,
  params = []
) => {
  const cefParams: CefParams = params.map((param) => {
    if (typeof param === "string") {
      return param;
    } else {
      const paramType = getWrfParamType(param, false);
      const [cefBuffer] = window.cefCreateArrayBuffer(param.length, paramType);
      // TODO(Dmitry): implement optimization to avoid copying when possible
      copyArrayToRustBuffer(param, cefBuffer, 0);
      return [cefBuffer, paramType];
    }
  });
  const returnParams = window.cefCallRustInSameThreadSync(name, cefParams);
  return transformReturnParams(returnParams);
};

export const wrfNewWorkerPort = (): MessagePort => {
  throw new Error("`wrfNewWorkerPort` is currently not supported on CEF");
};

export const serializeWrfArrayForPostMessage = (
  _postMessageData: Uint8Array
): PostMessageTypedArray => {
  throw new Error(
    "`serializeWrfArrayForPostMessage` is currently not supported on CEF"
  );
};

export const deserializeWrfArrayFromPostMessage = (
  _postMessageData: PostMessageTypedArray
): Uint8Array => {
  throw new Error(
    "`deserializeWrfArrayFromPostMessage` is currently not supported on CEF"
  );
};

export const initialize = (_initParams: unknown): Promise<void> =>
  new Promise<void>((resolve) => {
    overwriteTypedArraysWithWrfArrays();

    window.fromCefSetMouseCursor = (cursorId) => {
      if (document.body) {
        document.body.style.cursor = cursorMap[cursorId] || "default";
      }
    };

    window.fromCefCallJsFunction = (name, params) => {
      fromCefJsFunctions[name](transformReturnParams(params));
    };

    document.addEventListener("DOMContentLoaded", () => {
      const { showTextIME, textareaHasFocus } = makeTextarea(
        (taEvent: TextareaEvent) => {
          const slots = 20;
          const [buffer] = window.cefCreateArrayBuffer(
            slots * 4,
            WrfParamType.U8Buffer
          );
          const zerdeBuilder = new ZerdeBuilder({
            buffer,
            byteOffset: 0,
            slots,
            growCallback: () => {
              throw new Error("Growing of this buffer is not supported");
            },
          });

          if (taEvent.type === WorkerEvent.KeyDown) {
            zerdeKeyboardHandlers.keyDown(zerdeBuilder, taEvent);
          } else if (taEvent.type === WorkerEvent.KeyUp) {
            zerdeKeyboardHandlers.keyUp(zerdeBuilder, taEvent);
          } else if (taEvent.type === WorkerEvent.TextInput) {
            zerdeKeyboardHandlers.textInput(zerdeBuilder, taEvent);
          } else if (taEvent.type === WorkerEvent.TextCopy) {
            zerdeKeyboardHandlers.textCopy(zerdeBuilder);
          }

          window.cefHandleKeyboardEvent(buffer);
        }
      );

      window.fromCefSetIMEPosition = (x: number, y: number) => {
        showTextIME({ x, y });
      };

      document.addEventListener("keydown", (event) => {
        const code = event.keyCode;

        if (event.metaKey || event.ctrlKey) {
          if (!textareaHasFocus()) {
            // TODO(JP): Maybe at some point we should use some library for these keycodes,
            // e.g. see https://stackoverflow.com/questions/1465374/event-keycode-constants
            if (code == 67 /* c */) {
              window.cefTriggerCopy();
            } else if (code == 88 /* x */) {
              window.cefTriggerCut();
            } else if (code == 65 /* a */) {
              window.cefTriggerSelectAll();
            }
          }

          // We want pastes to also be triggered when the textarea has focus, so we can
          // handle the paste event in JS.
          if (code == 86 /* v */) {
            window.cefTriggerPaste();
          }
        }
      });

      resolve();
    });
  });

export const createBuffer: CreateBuffer = async (data) => {
  const paramType = getWrfParamType(data, false);
  const [cefBuffer] = window.cefCreateArrayBuffer(data.length, paramType);
  copyArrayToRustBuffer(data, cefBuffer, 0);
  return transformReturnParams([
    [cefBuffer, undefined, paramType],
  ])[0] as typeof data;
};

export const createReadOnlyBuffer: CreateBuffer = async (data) => {
  const paramType = getWrfParamType(data, true);
  const [cefBuffer, arcPtr] = window.cefCreateArrayBuffer(
    data.length,
    paramType
  );
  copyArrayToRustBuffer(data, cefBuffer, 0);
  return transformReturnParams([
    [cefBuffer, arcPtr, paramType],
  ])[0] as typeof data;
};
