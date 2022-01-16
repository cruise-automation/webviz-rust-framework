// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;
use wrflib_widget::*;

pub struct WidgetExampleApp {
    desktop_window: DesktopWindow,
    menu: Menu,
    button: Button,
    buttons: Vec<Button>,
}

impl WidgetExampleApp {
    pub fn new(_cx: &mut Cx) -> Self {
        Self {
            desktop_window: DesktopWindow::new(),
            button: Button::default(),
            buttons: (0..1000).map(|_| Button::default()).collect(),
            menu: Menu::main(vec![Menu::sub("Example", vec![Menu::line(), Menu::item("Quit Example", Cx::COMMAND_QUIT)])]),
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        self.desktop_window.handle(cx, event);

        if let ButtonEvent::Clicked = self.button.handle(cx, event) {
            log!("CLICKED Hello");
        }
        for (index, button) in self.buttons.iter_mut().enumerate() {
            if let ButtonEvent::Clicked = button.handle(cx, event) {
                log!("CLICKED {}", index);
            }
        }
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.desktop_window.begin_draw(cx, Some(&self.menu));
        cx.begin_wrapping_box();

        self.button.draw(cx, "Hello");
        for (index, button) in self.buttons.iter_mut().enumerate() {
            button.draw(cx, &format!("{}", index));
        }

        cx.end_wrapping_box();
        self.desktop_window.end_draw(cx);
    }
}

main_app!(WidgetExampleApp);
