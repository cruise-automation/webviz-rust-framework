// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! Performance profiling.

use crate::cx::*;

impl Cx {
    pub fn profile_start(&mut self, id: u64) {
        self.profiles.insert(id, UniversalInstant::now());
    }

    pub fn profile_end(&self, id: u64) {
        if let Some(inst) = self.profiles.get(&id) {
            log!("Profile {} time {}ms", id, inst.elapsed().as_millis());
        }
    }
}
