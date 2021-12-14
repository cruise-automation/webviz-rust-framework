// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib_cef_sys::cef_point_t;

#[derive(Clone, Debug)]
pub struct CefPoint {
    pub x: i32,
    pub y: i32,
}
impl CefPoint {
    #[allow(dead_code)]
    pub(crate) fn from_ptr(raw: *const cef_point_t) -> Self {
        Self::from(unsafe { &*raw })
    }
    #[allow(dead_code)]
    pub(crate) fn from(raw: &cef_point_t) -> Self {
        Self { x: raw.x, y: raw.y }
    }
}
impl Default for CefPoint {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}
