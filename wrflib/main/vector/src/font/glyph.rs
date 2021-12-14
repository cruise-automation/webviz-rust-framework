// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::font::{HorizontalMetrics, Outline};
use crate::geometry::Rectangle;

/// A glyph in a font.
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub horizontal_metrics: HorizontalMetrics,
    pub bounds: Rectangle,
    pub outline: Outline,
}
