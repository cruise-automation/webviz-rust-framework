// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;
use wrflib_widget::*;

const TEXT_HEADING: TextInsProps = TextInsProps {
    text_style: TextStyle { font_size: 28.0, line_spacing: 2.0, ..TEXT_STYLE_NORMAL },
    color: COLOR_WHITE,
    ..TextInsProps::DEFAULT
};
const TEXT_BODY: TextInsProps = TextInsProps {
    text_style: TextStyle { font_size: 14.0, height_factor: 2.0, line_spacing: 3.0, ..TEXT_STYLE_NORMAL },
    color: COLOR_WHITE,
    ..TextInsProps::DEFAULT
};

pub struct TourExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    next_button: NormalButton,
    back_button: NormalButton,
    slider: FloatSlider,
    font_size: f32,

    slider_r: FloatSlider,
    slider_g: FloatSlider,
    slider_b: FloatSlider,

    color: Vec4,
}

impl TourExampleApp {
    pub fn new(_cx: &mut Cx) -> Self {
        Self {
            window: Window { create_inner_size: Some(Vec2 { x: 700., y: 800. }), ..Window::default() },
            pass: Pass::default(),
            main_view: View::default(),
            next_button: NormalButton::default(),
            back_button: NormalButton::default(),
            slider: FloatSlider::new(),
            font_size: 14.,
            slider_r: FloatSlider::new(),
            slider_g: FloatSlider::new(),
            slider_b: FloatSlider::new(),
            color: COLOR_WHITE,
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        self.next_button.handle(cx, event);
        self.back_button.handle(cx, event);

        if let FloatSliderEvent::Change { scaled_value } = self.slider.handle(cx, event) {
            self.font_size = scaled_value;
            cx.request_draw();
        }

        if let FloatSliderEvent::Change { scaled_value } = self.slider_r.handle(cx, event) {
            self.color.x = scaled_value;
            cx.request_draw();
        }
        if let FloatSliderEvent::Change { scaled_value } = self.slider_g.handle(cx, event) {
            self.color.y = scaled_value;
            cx.request_draw();
        }
        if let FloatSliderEvent::Change { scaled_value } = self.slider_b.handle(cx, event) {
            self.color.z = scaled_value;
            cx.request_draw();
        }
    }

    pub fn draw_variable_text(&mut self, cx: &mut Cx) {
        TextIns::draw_walk(cx, "Text", &TEXT_HEADING);
        {
            // could be a text container at some point
            let text = cx.begin_turtle(Layout {
                walk: Walk { width: Width::Fill, height: Height::Compute, ..Walk::default() },
                // TODO: consider if we need those properties
                padding: Padding::all(10.),
                new_line_padding: 15.,
                line_wrap: LineWrap::Overflow,
                ..Layout::default()
            });
            TextIns::draw_walk(
                cx,
                "Text is probably the most essential widget for your UI. It will try to adapt to the dimensions of its \
                 container.\nYou can change its size:\n",
                &TEXT_BODY,
            );

            TextIns::draw_walk(
                cx,
                &format!("This text is {} pixels\n", self.font_size),
                // TODO: maybe introduce with_font_size()
                &TextInsProps { text_style: TextStyle { font_size: self.font_size, ..TEXT_BODY.text_style }, ..TEXT_BODY },
            );
            cx.end_turtle(text);
        }
        let row = cx.begin_row_turtle();

        let background_ranges = vec![
            FloatSliderBackgroundRange { min_scaled: 0.0, max_scaled: self.font_size, color: COLOR_BLUE800, height_pixels: 10. },
            FloatSliderBackgroundRange { min_scaled: self.font_size, max_scaled: 70.0, color: COLOR_WHITE, height_pixels: 10. },
        ];
        self.slider.draw(cx, self.font_size, 10.0, 70.0, Some(1.0), 1.0, Some(&background_ranges));
        cx.end_turtle(row);
    }

    pub fn draw_color_slider(slider: &mut FloatSlider, color: f32, width: f32, cx: &mut Cx) {
        let turtle = cx.begin_turtle(Layout { walk: Walk { width: Width::Fix(width), ..Walk::default() }, ..Layout::default() });
        let background_ranges = vec![
            FloatSliderBackgroundRange { min_scaled: 0.0, max_scaled: color, color: COLOR_BLUE800, height_pixels: 10. },
            FloatSliderBackgroundRange { min_scaled: color, max_scaled: 1.0, color: COLOR_WHITE, height_pixels: 10. },
        ];
        slider.draw(cx, color, 0.0, 1.0, Some(0.01), 1.0, Some(&background_ranges));
        cx.end_turtle(turtle);
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("333"));
        self.main_view.begin_view(cx, Layout { direction: Direction::Down, ..Layout::default() });

        // There should always be a outer turtle spanning the whole content as a first turtle
        let window_turtle = cx.begin_turtle(Layout {
            direction: Direction::Down,
            walk: Walk { width: Width::Fill, height: Height::Fill, ..Walk::default() },
            ..Layout::default()
        });

        {
            let top = cx.begin_turtle(Layout { walk: Walk::wh(Width::Fill, Height::Fix(27.)), ..Layout::default() });
            cx.end_turtle(top);
        }

        cx.begin_bottom_align();
        {
            let row = cx.begin_row_turtle();
            {
                self.back_button.draw(cx, "Back");

                cx.begin_right_align();
                self.next_button.draw(cx, "Next");
                cx.end_right_align();
            }
            cx.end_turtle(row);
        }
        cx.end_bottom_align();

        cx.begin_bottom_align();
        {
            {
                let row = cx.begin_row_turtle();
                TextIns::draw_walk(cx, "And its color:\n", &TEXT_BODY);

                TextIns::draw_walk(
                    cx,
                    &format!("Color: {{r: {:.2}, g: {:.2}, b: {:.2}}}\n", self.color.x, self.color.y, self.color.z),
                    &TextInsProps { color: self.color, ..TEXT_BODY },
                );
                cx.end_turtle(row);
            }
            {
                let row = cx.begin_row_turtle();
                let third_of_width = cx.get_width_left() / 3.;
                Self::draw_color_slider(&mut self.slider_r, self.color.x, third_of_width, cx);
                Self::draw_color_slider(&mut self.slider_g, self.color.y, third_of_width, cx);
                Self::draw_color_slider(&mut self.slider_b, self.color.z, third_of_width, cx);
                cx.end_turtle(row);
            }
        }
        cx.end_bottom_align();

        cx.begin_center_y_align();
        {
            self.draw_variable_text(cx);
        }
        cx.end_center_y_align();

        cx.end_turtle(window_turtle);

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(TourExampleApp);
