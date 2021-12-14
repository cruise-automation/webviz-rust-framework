// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::internal_iter::{InternalIterator, IntoInternalIterator};

/// A trait for extending a collection with each item of an internal iterator.
pub trait ExtendFromInternalIterator<T> {
    /// Extends `self` with each item of `internal_iter`.
    fn extend_from_internal_iter<I>(&mut self, internal_iter: I)
    where
        I: IntoInternalIterator<Item = T>;
}

impl<T> ExtendFromInternalIterator<T> for Vec<T> {
    fn extend_from_internal_iter<I>(&mut self, internal_iter: I)
    where
        I: IntoInternalIterator<Item = T>,
    {
        internal_iter.into_internal_iter().for_each(&mut |item| {
            self.push(item);
            true
        });
    }
}
