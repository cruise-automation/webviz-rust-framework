// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib_cef_sys::cef_size_t;

#[derive(Clone, Debug)]
pub struct CefSize {
    pub width: i32,
    pub height: i32,
}
impl CefSize {
    pub(crate) fn from_ptr(raw: *const cef_size_t) -> Self {
        Self::from(unsafe { &*raw })
    }
    pub(crate) fn from(raw: &cef_size_t) -> Self {
        CefSize { width: raw.width, height: raw.height }
    }
}
impl Default for CefSize {
    fn default() -> Self {
        Self { width: 0, height: 0 }
    }
}
