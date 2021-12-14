// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

export type RpcEvent = {
  button?: number;
  pageX?: number;
  pageY?: number;
  timeStamp?: number;
  shiftKey?: boolean;
  ctrlKey?: boolean;
  altKey?: boolean;
  metaKey?: boolean;
  deltaMode?: number;
  deltaX?: number;
  deltaY?: number;
  wheelDeltaY?: number;
  keyCode?: number;
  charCode?: number;
  repeat?: boolean;
};

// Turn common types of events into a regular object.
export function makeRpcEvent(event: Event): RpcEvent {
  return {
    button: event["button"],
    pageX: event["pageX"],
    pageY: event["pageY"],
    timeStamp: event["timeStamp"],
    shiftKey: event["shiftKey"],
    ctrlKey: event["ctrlKey"],
    altKey: event["altKey"],
    metaKey: event["metaKey"],
    deltaMode: event["deltaMode"],
    deltaX: event["deltaX"],
    deltaY: event["deltaY"],
    wheelDeltaY: event["wheelDeltaY"],
    keyCode: event["keyCode"],
    charCode: event["charCode"],
    repeat: event["repeat"],
  };
}
