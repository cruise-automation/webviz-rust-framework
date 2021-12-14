// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::geometry::{Point, Vector};

/// A trait for transformations in 2-dimensional Euclidian space.
pub trait Transformation {
    /// Applies `self` to the given [`Point`].
    fn transform_point(&self, point: Point) -> Point;

    /// Applies `self` to the given [`Vector`].
    fn transform_vector(&self, vector: Vector) -> Vector;
}
