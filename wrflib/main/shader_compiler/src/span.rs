// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

#[derive(Clone, Copy, Debug, Default, Hash, Eq, Ord, PartialOrd, PartialEq)]
pub struct CodeFragmentId(pub usize);

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialOrd, PartialEq)]
pub struct Span {
    pub code_fragment_id: CodeFragmentId,
    pub start: usize,
    pub end: usize,
}
