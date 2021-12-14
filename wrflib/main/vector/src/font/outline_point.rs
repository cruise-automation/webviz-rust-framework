// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::geometry::{Point, Transform, Transformation};

/// A point in an outline.
///
/// An outline point is either on the curve or off the curve. If it is on the curve, it represents
/// an endpoint of a quadratic b-spline curve segment. Otherwise, it represents a control point of
/// a quadratic b-spline curve segment. Each quadratic b-spline curve segment has two endpoints and
/// zero or more control points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutlinePoint {
    pub is_on_curve: bool,
    pub point: Point,
}

impl Transform for OutlinePoint {
    fn transform<T>(self, t: &T) -> OutlinePoint
    where
        T: Transformation,
    {
        OutlinePoint { is_on_curve: self.is_on_curve, point: self.point.transform(t) }
    }

    fn transform_mut<T>(&mut self, t: &T)
    where
        T: Transformation,
    {
        *self = self.transform(t)
    }
}
