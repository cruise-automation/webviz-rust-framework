// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Default for Axis {
    fn default() -> Self {
        Axis::Horizontal
    }
}
