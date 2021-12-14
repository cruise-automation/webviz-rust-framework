// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::internal_iter::InternalIterator;
use crate::path::LinePathCommand;

/// An extension trait for iterators over line path commands.
pub trait LinePathIterator: InternalIterator<Item = LinePathCommand> {}

impl<I> LinePathIterator for I where I: InternalIterator<Item = LinePathCommand> {}
