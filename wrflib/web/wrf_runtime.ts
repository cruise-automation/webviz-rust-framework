// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// This is the universal Wrf Runtime which will work on both CEF and WebAssembly environments,
// doing runtime detection of which modules to load. No other file besides this one should conditionally
// branch based on environments, such that cef/wasm runtimes can work without including unnecessary code.

import * as wasm from "./wrf_wasm_runtime";
import * as cef from "./wrf_cef_runtime";
import { jsRuntime } from "./type_of_runtime";
import "./wrf.css";

const {
  initialize,
  wrfNewWorkerPort,
  registerCallJsCallbacks,
  unregisterCallJsCallbacks,
  callRust,
  serializeWrfArrayForPostMessage,
  deserializeWrfArrayFromPostMessage,
  callRustInSameThreadSync,
  createBuffer,
  createReadOnlyBuffer,
} = jsRuntime === "cef" ? cef : wasm;

export {
  initialize,
  wrfNewWorkerPort,
  registerCallJsCallbacks,
  unregisterCallJsCallbacks,
  callRust,
  serializeWrfArrayForPostMessage,
  deserializeWrfArrayFromPostMessage,
  callRustInSameThreadSync,
  jsRuntime,
  createBuffer,
  createReadOnlyBuffer,
};
