// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib_cef::CefColor;
use wrflib_shader_compiler::math::Vec4;

pub fn vec4_to_cef_color(color: &Vec4) -> CefColor {
    fn normalize_to_u8(value: f32) -> u8 {
        (value * 255.0) as u8
    }
    CefColor::from_argb(normalize_to_u8(color.w), normalize_to_u8(color.x), normalize_to_u8(color.y), normalize_to_u8(color.z))
}
