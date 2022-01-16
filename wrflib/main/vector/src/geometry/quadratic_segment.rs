// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::geometry::{Point, Transform, Transformation};
use crate::internal_iter::InternalIterator;

/// A quadratic bezier curve segment in 2-dimensional Euclidian space.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub(crate) struct QuadraticSegment {
    pub(crate) p0: Point,
    pub(crate) p1: Point,
    pub(crate) p2: Point,
}

impl QuadraticSegment {
    /// Creates a new quadratic bezier curve segment with the given control points.
    pub(crate) fn new(p0: Point, p1: Point, p2: Point) -> QuadraticSegment {
        QuadraticSegment { p0, p1, p2 }
    }

    /// Returns true if `self` is approximately linear with tolerance `epsilon`.
    pub(crate) fn is_approximately_linear(self, epsilon: f32) -> bool {
        debug_assert!(!epsilon.is_nan());
        debug_assert!(!self.p0.x.is_nan());
        debug_assert!(!self.p0.y.is_nan());
        debug_assert!(!self.p1.x.is_nan());
        debug_assert!(!self.p1.y.is_nan());
        let v1 = self.p1 - self.p0;
        (if let Some(vx) = (self.p2 - self.p0).normalize() {
            // If the baseline is a line segment, the segment is approximately linear if the
            // rejection of the control point from the baseline is less than `epsilon`.
            v1.cross(vx).abs()
        } else {
            // If the baseline is a single point, the segment is approximately linear if the
            // distance of the control point from the baseline is less than `epsilon`.
            v1.length()
        }) < epsilon
    }

    /// Splits `self` into two quadratic Bezier curve segments, at parameter `t`.
    pub(crate) fn split(self, t: f32) -> (QuadraticSegment, QuadraticSegment) {
        let p01 = self.p0.lerp(self.p1, t);
        let p12 = self.p1.lerp(self.p2, t);
        let p012 = p01.lerp(p12, t);
        (QuadraticSegment::new(self.p0, p01, p012), QuadraticSegment::new(p012, p12, self.p2))
    }

    /// Returns an iterator over the points of a polyline that approximates `self` with tolerance
    /// `epsilon`, *excluding* the first point.
    pub(crate) fn linearize(self, epsilon: f32) -> Linearize {
        Linearize { segment: self, epsilon }
    }
}

impl Transform for QuadraticSegment {
    fn transform<T>(self, t: &T) -> QuadraticSegment
    where
        T: Transformation,
    {
        QuadraticSegment::new(self.p0.transform(t), self.p1.transform(t), self.p2.transform(t))
    }

    fn transform_mut<T>(&mut self, t: &T)
    where
        T: Transformation,
    {
        *self = self.transform(t);
    }
}

/// An iterator over the points of a polyline that approximates `self` with tolerance `epsilon`,
/// *excluding* the first point.
#[derive(Clone, Copy)]
pub struct Linearize {
    segment: QuadraticSegment,
    epsilon: f32,
}

impl InternalIterator for Linearize {
    type Item = Point;

    fn for_each<F>(self, f: &mut F) -> bool
    where
        F: FnMut(Point) -> bool,
    {
        if self.segment.is_approximately_linear(self.epsilon) {
            return f(self.segment.p2);
        }
        let (segment_0, segment_1) = self.segment.split(0.5);
        if !segment_0.linearize(self.epsilon).for_each(f) {
            return false;
        }
        segment_1.linearize(self.epsilon).for_each(f)
    }
}
