// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;

#[derive(Default)]
struct App {}

impl App {
    fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    fn handle(&mut self, _cx: &mut Cx, event: &mut Event) {
        match event {
            Event::Construct => {
                log!("Hello, world!");
            }
            _ => {}
        }
    }
    fn draw(&mut self, _cx: &mut Cx) {}
}

main_app!(App);
