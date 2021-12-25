// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// Clippy TODO
#![warn(clippy::module_inception)]

pub mod outline;

mod font;
mod glyph;
mod horizontal_metrics;
mod outline_point;

pub use self::font::VectorFont;
pub use self::glyph::Glyph;
pub use self::horizontal_metrics::HorizontalMetrics;
pub use self::outline::Outline;
pub(crate) use self::outline_point::OutlinePoint;
