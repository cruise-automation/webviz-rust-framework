// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::geometry::Transformation;

/// A trait to transform geometric objects in 2-dimensional Euclidian space.
pub trait Transform {
    #[must_use]
    fn transform<T>(self, t: &T) -> Self
    where
        T: Transformation;

    fn transform_mut<T>(&mut self, t: &T)
    where
        T: Transformation;
}
