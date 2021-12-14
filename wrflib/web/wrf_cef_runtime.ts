// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import { cursorMap } from "./cursor_map";
import { copyUint8ArrayToRustBuffer } from "./common";
import { makeTextarea, TextareaEvent } from "./make_textarea";
import {
  CallRust,
  CallJsCallback,
  CallRustInSameThreadSync,
  WrfParam,
  PostMessageTypedArray,
} from "./types";
import {
  getCachedUint8Buffer,
  overwriteTypedArraysWithWrfArrays,
  isWrfBuffer,
  wrfArrayCoversWrfBuffer,
  getWrfBufferCef,
  WrfBuffer,
} from "./wrf_buffer";
import { ZerdeBuilder } from "./zerde";
import { zerdeKeyboardHandlers } from "./zerde_keyboard_handlers";

type CefParams = (string | [ArrayBuffer, boolean])[];

declare global {
  interface Window {
    // Defined externally in `cef_browser.rs`.
    cefCallRust: (name: string, params: CefParams, callbackId: number) => void;
    cefCallRustInSameThreadSync: (
      name: string,
      params: CefParams
    ) => (string | [ArrayBuffer, number])[];
    cefReadyForMessages: () => void;
    cefCreateArrayBuffer: (size: number) => ArrayBuffer;
    cefHandleKeyboardEvent: (buffer: ArrayBuffer) => void;
    cefTriggerCut: () => void;
    cefTriggerCopy: () => void;
    cefTriggerPaste: () => void;
    cefTriggerSelectAll: () => void;

    fromCefSetMouseCursor: (cursor: number) => void;
    fromCefSetIMEPosition: (x: number, y: number) => void;
    fromCefCallJsFunction: (
      name: string,
      params: (string | [ArrayBuffer, number | undefined])[]
    ) => void;
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
        if (!wrfArrayCoversWrfBuffer(param)) {
          throw new Error(
            "callRust only supports buffers that span the entire underlying WrfBuffer"
          );
        }
        const wrfBuffer = param.buffer as WrfBuffer;
        return [wrfBuffer.__wrflibWasmBuffer, wrfBuffer.readonly];
      }
      const cefBuffer = window.cefCreateArrayBuffer(param.byteLength);
      copyUint8ArrayToRustBuffer(param, cefBuffer, 0);
      return [cefBuffer, false];
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

// TODO(JP): Some of this code is duplicated with callRust/call_js; see if we can reuse some.
export const callRustInSameThreadSync: CallRustInSameThreadSync = (
  name,
  params = []
) => {
  const cefParams: CefParams = params.map((param) => {
    if (typeof param === "string") {
      return param;
    } else {
      const cefBuffer = window.cefCreateArrayBuffer(param.byteLength);
      // TODO(Dmitry): implement optimization to avoid copying when possible
      copyUint8ArrayToRustBuffer(param, cefBuffer, 0);
      return [cefBuffer, false];
    }
  });
  const returnParams = window.cefCallRustInSameThreadSync(name, cefParams);
  return returnParams.map((param) => {
    if (typeof param === "string") {
      return param;
    } else {
      const [buffer, arcPtr] = param;
      const wrfBuffer = getWrfBufferCef(buffer, arcPtr);
      // Creating Uint8Array with stable identity as that's what underlying underlying API expects
      const uint8Buffer = getCachedUint8Buffer(
        wrfBuffer,
        new Uint8Array(wrfBuffer)
      );
      return uint8Buffer;
    }
  });
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

export const initialize = (): Promise<void> =>
  new Promise<void>((resolve) => {
    overwriteTypedArraysWithWrfArrays();

    window.fromCefSetMouseCursor = (cursorId) => {
      if (document.body) {
        document.body.style.cursor = cursorMap[cursorId] || "default";
      }
    };

    window.fromCefCallJsFunction = (name, params) => {
      const transformedParams = params.map((param) => {
        if (typeof param === "string") {
          return param;
        } else {
          const [buffer, arcPtr] = param;
          const wrfBuffer = getWrfBufferCef(buffer, arcPtr);
          // Creaing Uint8Array with stable identity as that's what underlying underlying API expects
          const uint8Buffer = getCachedUint8Buffer(
            wrfBuffer,
            // This actually creates a WrfUint8Array as this was overwritten above in overwriteTypedArraysWithWrfArrays()
            new Uint8Array(wrfBuffer)
          );
          return uint8Buffer;
        }
      });
      fromCefJsFunctions[name](transformedParams);
    };

    document.addEventListener("DOMContentLoaded", () => {
      const { showTextIME, textareaHasFocus } = makeTextarea(
        (taEvent: TextareaEvent) => {
          const slots = 20;
          const buffer = window.cefCreateArrayBuffer(slots * 4);
          const zerdeBuilder = new ZerdeBuilder({
            buffer,
            byteOffset: 0,
            slots,
            growCallback: () => {
              throw new Error("Growing of this buffer is not supported");
            },
          });

          if (taEvent.type === "KeyDown") {
            zerdeKeyboardHandlers.keyDown(zerdeBuilder, taEvent);
          } else if (taEvent.type === "KeyUp") {
            zerdeKeyboardHandlers.keyUp(zerdeBuilder, taEvent);
          } else if (taEvent.type === "TextInput") {
            zerdeKeyboardHandlers.textInput(zerdeBuilder, taEvent);
          } else if (taEvent.type === "TextCopy") {
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
