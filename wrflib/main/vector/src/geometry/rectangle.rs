// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::geometry::{Point, Transform, Transformation};

/// An axis-aligned rectangle in 2-dimensional Euclidian space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Rectangle {
    pub p_min: Point,
    pub p_max: Point,
}

impl Rectangle {
    /// Creates a new rectangle with the given minimum and maximum point.
    pub(crate) fn new(p_min: Point, p_max: Point) -> Rectangle {
        Rectangle { p_min, p_max }
    }
}

impl Transform for Rectangle {
    fn transform<T>(self, t: &T) -> Rectangle
    where
        T: Transformation,
    {
        Rectangle::new(self.p_min.transform(t), self.p_max.transform(t))
    }

    fn transform_mut<T>(&mut self, t: &T)
    where
        T: Transformation,
    {
        *self = self.transform(t);
    }
}
