// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;
use wrflib_widget::*;

pub struct PopoverExampleApp {
    desktop_window: DesktopWindow,
    menu: Menu,
    button: Button,
    popover: Option<Popover>,
}

impl PopoverExampleApp {
    pub fn new(_cx: &mut Cx) -> Self {
        Self {
            desktop_window: DesktopWindow::new(),
            button: Button::default(),
            menu: Menu::main(vec![Menu::sub("Example", vec![Menu::line(), Menu::item("Quit Example", Cx::COMMAND_QUIT)])]),
            popover: None,
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let Some(popover) = &mut self.popover {
            popover.handle(cx, event);
        }

        self.desktop_window.handle(cx, event);

        if let ButtonEvent::Clicked = self.button.handle(cx, event) {
            self.popover = if self.popover.is_none() { Some(Popover::default()) } else { None };
            cx.request_draw();
        }
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.desktop_window.begin_draw(cx, Some(&self.menu));

        // Popover currently only supports drawing above the current box position,
        // so make some space above it.
        cx.move_draw_pos(0., 200.);

        if let Some(popover) = &mut self.popover {
            popover.begin_draw(cx, Width::Compute, Height::Compute, COLOR_BLACK);
            cx.begin_padding_box(Padding::all(10.));
            TextIns::draw_walk(cx, "hello!", &TextInsProps::DEFAULT);
            cx.end_padding_box();
            popover.end_draw(cx);
        }
        self.button.draw(cx, "Hello");

        self.desktop_window.end_draw(cx);
    }
}

main_app!(PopoverExampleApp);
