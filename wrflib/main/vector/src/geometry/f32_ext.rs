// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

/// An extension trait for [`f32`].
pub trait F32Ext {
    /// Linearly interpolate between `self` and `other` with parameter `t`.
    fn lerp(self, other: f32, t: f32) -> f32;
}

impl F32Ext for f32 {
    fn lerp(self, other: f32, t: f32) -> f32 {
        self * (1.0 - t) + other * t
    }
}
