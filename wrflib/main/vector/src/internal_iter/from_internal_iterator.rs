// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::internal_iter::{ExtendFromInternalIterator, IntoInternalIterator};

/// A trait for conversion from an internal iterator.
///
/// This trait is commonly implemented for collections. It is useful when you have an internal
/// iterator but you need a collection.
pub trait FromInternalIterator<T> {
    /// Creates [`Self`] from an internal iterator.
    ///
    /// Note that `from_internal_iter` is almost never used directly. Instead, it is used by
    /// calling [`crate::internal_iter::InternalIterator::collect`].
    fn from_internal_iter<I>(internal_iter: I) -> Self
    where
        I: IntoInternalIterator<Item = T>;
}

impl<T> FromInternalIterator<T> for Vec<T> {
    fn from_internal_iter<I>(internal_iter: I) -> Self
    where
        I: IntoInternalIterator<Item = T>,
    {
        let mut vec = Vec::new();
        vec.extend_from_internal_iter(internal_iter);
        vec
    }
}
