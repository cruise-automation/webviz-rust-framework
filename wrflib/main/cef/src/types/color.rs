// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib_cef_sys::cef_color_t;

// cef_color_t is just a u32 so wrapping this into a struct to achieve strong typing
// see New Type Idiom: https://doc.rust-lang.org/rust-by-example/generics/new_types.html
#[derive(Clone, Copy, Debug)]
pub struct CefColor(cef_color_t);

impl CefColor {
    pub fn from_argb(a: u8, r: u8, g: u8, b: u8) -> CefColor {
        CefColor((a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32)
    }

    pub fn from_u32(color: u32) -> CefColor {
        CefColor(color)
    }

    pub fn to_cef(&self) -> cef_color_t {
        self.0
    }
}
