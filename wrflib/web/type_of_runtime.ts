// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// We only define `cefCallRust` if in CEF, so we can use this for environment detection.
// This should only be used at the top level `wrflib_runtime` file or in test, since we want to keep
// CEF and WASM code separate for bundle size.
export const jsRuntime = "cefCallRust" in self ? "cef" : "wasm";

// Whether or not we're in a WebWorker.
// From https://stackoverflow.com/a/23619712
export const inWorker = typeof importScripts === "function";
