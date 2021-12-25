// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

/// The horizontal metrics for a glyph
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HorizontalMetrics {
    pub advance_width: f32,
    pub(crate) left_side_bearing: f32,
}
