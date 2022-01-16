// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import { RpcKeyboardEvent, makeRpcKeyboardEvent } from "./make_rpc_event";
import { WorkerEvent } from "./rpc_types";

export type TextareaEventKeyDown = {
  type: WorkerEvent.KeyDown;
  event: RpcKeyboardEvent;
};
export type TextareaEventKeyUp = {
  type: WorkerEvent.KeyUp;
  event: RpcKeyboardEvent;
};
export type TextareaEventTextInput = {
  type: WorkerEvent.TextInput;
  wasPaste: boolean;
  input: string;
  replaceLast: boolean;
};
export type TextareaEventTextCopy = {
  type: WorkerEvent.TextCopy;
};
export type TextareaEvent =
  | TextareaEventKeyDown
  | TextareaEventKeyUp
  | TextareaEventTextInput
  | TextareaEventTextCopy;

// Create a hidden textarea which is purely used for text input into Rust.
export function makeTextarea(callback: (taEvent: TextareaEvent) => void): {
  showTextIME: (pos: { x: number; y: number }) => void;
  textareaHasFocus: () => boolean;
} {
  let ta: HTMLTextAreaElement;

  // NOTE(JP): This looks a bit convoluted, but it's the most reliable method I could find to return the focus to the textarea!
  function fixFocus() {
    setTimeout(() => {
      if (
        ta &&
        document.activeElement !== ta &&
        !document.getElementById("js_root")?.contains(document.activeElement)
      ) {
        ta.focus();
      }
    });
  }
  document.addEventListener("mousedown", fixFocus, true);
  document.addEventListener("mouseup", fixFocus, true);
  document.addEventListener("focus", fixFocus, true);
  document.addEventListener("blur", fixFocus, true);

  let textAreaPos: { x: number; y: number } | undefined;
  const updateTextAreaPos = () => {
    if (!textAreaPos) {
      ta.style.left = -100 + "px";
      ta.style.top = -100 + "px";
    } else {
      ta.style.left = Math.round(textAreaPos.x) - 4 + "px";
      ta.style.top = Math.round(textAreaPos.y) + "px";
    }
  };

  function showTextIME({ x, y }: { x: number; y: number }) {
    textAreaPos = { x, y };
    updateTextAreaPos();
  }

  let wasPaste = false;
  let lastLen = 0;
  let uglyIMEHack = false;

  const recreateTextarea = function () {
    if (ta) document.body.removeChild(ta);

    ta = document.createElement("textarea");
    ta.className = "cx_webgl_textinput";
    ta.setAttribute("autocomplete", "off");
    ta.setAttribute("autocorrect", "off");
    ta.setAttribute("autocapitalize", "off");
    ta.setAttribute("spellcheck", "false");
    const style = document.createElement("style");
    style.innerHTML = `
      textarea.cx_webgl_textinput {
        z-index: 1000;
        position: absolute;
        opacity: 0;
        border-radius: 4px;
        color:white;
        font-size: 6;
        background: gray;
        -moz-appearance: none;
        appearance:none;
        border:none;
        resize: none;
        outline: none;
        overflow: hidden;
        text-indent: 0px;
        padding: 0 0px;
        margin: 0 -1px;
        text-indent: 0px;
        -ms-user-select: text;
        -moz-user-select: text;
        -webkit-user-select: text;
        user-select: text;
        white-space: pre!important;
      }
      textarea:focus.cx_webgl_textinput {
        outline: 0px !important;
        -webkit-appearance: none;
      }
    `;
    document.body.appendChild(style);
    ta.style.left = -100 + "px";
    ta.style.top = -100 + "px";
    ta.style.height = 1 + "px";
    ta.style.width = 1 + "px";

    ta.addEventListener("contextmenu", (event) => {
      event.preventDefault();
      return false;
    });
    document.body.appendChild(ta);
    ta.focus();
    updateTextAreaPos();

    ta.addEventListener("cut", () => {
      setTimeout(() => {
        ta.value = "";
        lastLen = 0;
      });
    });
    ta.addEventListener("copy", () => {
      setTimeout(() => {
        ta.value = "";
        lastLen = 0;
      });
    });
    ta.addEventListener("paste", () => {
      wasPaste = true;
    });

    ta.addEventListener("input", () => {
      if (ta.value.length > 0) {
        if (wasPaste) {
          wasPaste = false;
          const input = ta.value.substring(lastLen);
          ta.value = "";

          callback({
            type: WorkerEvent.TextInput,
            wasPaste: true,
            input,
            replaceLast: false,
          });
        } else {
          let replaceLast = false;
          let textValue = ta.value;
          if (ta.value.length >= 2) {
            // we want the second char
            textValue = ta.value.substring(1, 2);
            ta.value = textValue;
          } else if (ta.value.length == 1 && lastLen == ta.value.length) {
            // its an IME replace
            replaceLast = true;
          }
          // we should send a replace last
          if (replaceLast || textValue != "\n") {
            callback({
              type: WorkerEvent.TextInput,
              wasPaste: false,
              input: textValue,
              replaceLast: replaceLast,
            });
          }
        }
      }
      lastLen = ta.value.length;
    });

    ta.addEventListener("keydown", (event) => {
      const code = event.keyCode;

      if (code == 18 || code == 17 || code == 16) event.preventDefault(); // alt
      if (code === 8 || code === 9) event.preventDefault(); // backspace/tab
      if (code === 89 && (event.metaKey || event.ctrlKey))
        event.preventDefault(); // all (select all)
      if (code === 83 && (event.metaKey || event.ctrlKey))
        event.preventDefault(); // ctrl s
      if (code >= 33 && code <= 40) {
        // if we are using arrow keys, home or end
        ta.value = "";
        lastLen = ta.value.length;
      }
      if ((code === 88 || code == 67) && (event.metaKey || event.ctrlKey)) {
        // copy or cut
        // we need to request the clipboard
        callback({ type: WorkerEvent.TextCopy });
        event.preventDefault();
      }
      if (code === 90 && (event.metaKey || event.ctrlKey)) {
        // ctrl/cmd + z
        updateTextAreaPos();
        ta.value = "";
        uglyIMEHack = true;
        ta.readOnly = true;
        event.preventDefault();
      }

      callback({
        type: WorkerEvent.KeyDown,
        event: makeRpcKeyboardEvent(event),
      });
    });

    ta.addEventListener("keyup", (event) => {
      const code = event.keyCode;
      if (code == 18 || code == 17 || code == 16) event.preventDefault(); // alt
      if (code == 91) event.preventDefault(); // left window key
      if (uglyIMEHack) {
        uglyIMEHack = false;
        recreateTextarea();
      }

      callback({
        type: WorkerEvent.KeyUp,
        event: makeRpcKeyboardEvent(event),
      });
    });
  };
  recreateTextarea();

  function textareaHasFocus(): boolean {
    return document.activeElement == ta;
  }

  return { showTextIME, textareaHasFocus };
}
