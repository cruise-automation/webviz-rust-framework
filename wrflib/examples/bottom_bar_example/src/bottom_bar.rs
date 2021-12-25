// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;
use wrflib_widget::*;

pub struct BottomBar {
    switch_button: NormalButton,
    token_input: TextInput,
    play_speed_button: NormalButton,
    slider: FloatSlider,

    play_speed: String,
    position: f32,
}

impl BottomBar {
    pub fn new() -> Self {
        Self {
            switch_button: NormalButton::default(),
            token_input: TextInput::new(TextInputOptions {
                empty_message: "Enter token".to_string(),
                ..TextInputOptions::default()
            }),
            play_speed_button: NormalButton::default(),
            slider: FloatSlider::new(),
            play_speed: String::from("1x"),
            position: 0.5,
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let ButtonEvent::Clicked = self.switch_button.handle(cx, event) {
            log!("button clicked!");
        }
        if let ButtonEvent::Clicked = self.play_speed_button.handle(cx, event) {
            if self.play_speed == "1x" {
                self.play_speed = String::from("Fixed: 33ms");
            } else {
                self.play_speed = String::from("1x");
            }
            cx.request_draw();
        }
        if let FloatSliderEvent::Change { scaled_value } = self.slider.handle(cx, event) {
            self.position = scaled_value;
            cx.request_draw();
        }

        self.token_input.handle(cx, event);
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        let outer_turtle = cx.begin_turtle(Layout { direction: Direction::Down, ..Layout::default() });

        cx.begin_bottom_align();
        let bottom_bar_turtle = cx.begin_turtle(Layout {
            direction: Direction::Right,
            walk: Walk { width: Width::Fill, height: Height::Compute },
            ..Layout::default()
        });
        {
            self.switch_button.draw(cx, "*");

            self.token_input.draw(cx);

            cx.begin_right_align();
            self.play_speed_button.draw(cx, &self.play_speed);
            cx.end_right_align();

            self.slider.draw(cx, self.position, 0.0, 1.0, None, 1.0, None);
        }
        cx.end_turtle(bottom_bar_turtle);
        cx.end_bottom_align();

        cx.end_turtle(outer_turtle);
    }
}
