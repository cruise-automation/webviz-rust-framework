// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib_cef_sys::cef_range_t;

#[derive(Clone, Debug)]
pub struct CefRange {
    pub from: i32,
    pub to: i32,
}
impl CefRange {
    pub(crate) fn from_ptr(raw: *const cef_range_t) -> Self {
        Self::from(unsafe { &*raw })
    }
    pub(crate) fn from(raw: &cef_range_t) -> Self {
        Self { from: raw.from, to: raw.to }
    }
}
impl Default for CefRange {
    fn default() -> Self {
        Self { from: 0, to: 0 }
    }
}
