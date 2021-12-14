// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! Framework-specific code that runs on startup.

use crate::cx::*;

impl Cx {
    pub fn app_load(&mut self) {
        crate::geometry::generate_default_geometries(self);
    }
}
