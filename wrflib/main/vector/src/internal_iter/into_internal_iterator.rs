// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::internal_iter::InternalIterator;

/// A trait for conversion into an internal iterator.
///
/// This trait is commonly implemented for collections. It is useful when you want to write a
/// function that can take either a collection or an internal iterator as input.
pub trait IntoInternalIterator {
    type Item;
    type IntoInternalIter: InternalIterator<Item = Self::Item>;

    /// Converts `self` into an internal iterator.
    fn into_internal_iter(self) -> Self::IntoInternalIter;
}

impl<I> IntoInternalIterator for I
where
    I: InternalIterator,
{
    type Item = I::Item;
    type IntoInternalIter = I;

    fn into_internal_iter(self) -> Self::IntoInternalIter {
        self
    }
}
