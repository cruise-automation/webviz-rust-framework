// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::geometry::{Point, Transform, Transformation};

/// A command in a path
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadraticTo(Point, Point),
    Close,
}

impl Transform for PathCommand {
    fn transform<T>(self, t: &T) -> PathCommand
    where
        T: Transformation,
    {
        match self {
            PathCommand::MoveTo(p) => PathCommand::MoveTo(p.transform(t)),
            PathCommand::LineTo(p) => PathCommand::LineTo(p.transform(t)),
            PathCommand::QuadraticTo(p1, p) => PathCommand::QuadraticTo(p1.transform(t), p.transform(t)),
            PathCommand::Close => PathCommand::Close,
        }
    }

    fn transform_mut<T>(&mut self, t: &T)
    where
        T: Transformation,
    {
        *self = self.transform(t);
    }
}
