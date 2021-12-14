// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import { RpcEvent } from "./make_rpc_event";
import {
  TextareaEventKeyDown,
  TextareaEventKeyUp,
  TextareaEventTextInput,
} from "./make_textarea";
import { ZerdeBuilder } from "./zerde";

export function packKeyModifier(e: RpcEvent): number {
  return (
    (e.shiftKey ? 1 : 0) |
    (e.ctrlKey ? 2 : 0) |
    (e.altKey ? 4 : 0) |
    (e.metaKey ? 8 : 0)
  );
}

export const zerdeKeyboardHandlers = {
  keyDown(zerdeBuilder: ZerdeBuilder, data: TextareaEventKeyDown): void {
    zerdeBuilder.sendU32(12);
    zerdeBuilder.sendU32(data.event.keyCode);
    zerdeBuilder.sendU32(data.event.repeat ? 1 : 0);
    zerdeBuilder.sendU32(packKeyModifier(data.event));
    zerdeBuilder.sendF64(performance.now() / 1000.0);
  },

  keyUp(zerdeBuilder: ZerdeBuilder, data: TextareaEventKeyUp): void {
    zerdeBuilder.sendU32(13);
    zerdeBuilder.sendU32(data.event.keyCode);
    zerdeBuilder.sendU32(data.event.repeat ? 1 : 0);
    zerdeBuilder.sendU32(packKeyModifier(data.event));
    zerdeBuilder.sendF64(performance.now() / 1000.0);
  },

  textInput(zerdeBuilder: ZerdeBuilder, data: TextareaEventTextInput): void {
    zerdeBuilder.sendU32(14);
    zerdeBuilder.sendU32(data.wasPaste ? 1 : 0),
      zerdeBuilder.sendU32(data.replaceLast ? 1 : 0),
      zerdeBuilder.sendString(data.input);
  },

  textCopy(zerdeBuilder: ZerdeBuilder): void {
    zerdeBuilder.sendU32(17);
  },
};
