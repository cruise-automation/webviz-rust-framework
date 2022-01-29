// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// a bunch o buttons to select the world
use wrflib::*;
use wrflib_components::*;

#[derive(Default)]
pub struct FieldWorld {
    pub area: Area,
}

impl FieldWorld {
    pub fn handle(&mut self, _cx: &mut Cx, _event: &mut Event) {
        // lets see.
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        SkyBox::draw(cx, vec3(0., 0., 0.));
    }
}
