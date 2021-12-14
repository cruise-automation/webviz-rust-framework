// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

/// A trapezoid in 2-dimensional Euclidian space.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Trapezoid {
    pub xs: [f32; 2],
    pub ys: [f32; 4],
}
