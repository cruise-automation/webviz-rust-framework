// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;

mod single_button;
use single_button::*;

#[derive(Default)]
pub struct SingleButtonExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    single_button: SingleButton,
}

impl SingleButtonExampleApp {
    pub fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        self.single_button.handle_single_button(cx, event);
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("300"));
        self.main_view.begin_view(cx, Layout { direction: Direction::Down, padding: Padding::top(30.), ..Layout::default() });

        self.single_button.draw_single_button(cx);

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(SingleButtonExampleApp);
