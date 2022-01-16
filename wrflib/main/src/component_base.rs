// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct ComponentId(u64);

#[derive(Default, Debug)]
pub struct ComponentBase {
    pub(crate) id: Option<ComponentId>,
    pub(crate) area: Area,
}

impl ComponentBase {
    pub fn register_component_area(&mut self, cx: &mut Cx, area: Area) {
        if self.id.is_none() {
            self.id = Some(ComponentId(cx.next_component_id));
            cx.next_component_id += 1;
        }
        self.area = area;
    }
}
