// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;

mod bottom_bar;
use bottom_bar::*;

pub struct BottomBarExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    bottom_bar: BottomBar,
}

impl BottomBarExampleApp {
    pub fn new(_cx: &mut Cx) -> Self {
        Self {
            window: Window { create_inner_size: Some(Vec2 { x: 700., y: 400. }), ..Window::default() },
            pass: Pass::default(),
            bottom_bar: BottomBar::new(),
            main_view: View::default(),
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        self.bottom_bar.handle(cx, event);
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("333"));
        self.main_view.begin_view(cx, Layout { direction: Direction::Down, ..Layout::default() });

        self.bottom_bar.draw(cx);

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(BottomBarExampleApp);
