// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;
use wrflib_widget::*;

#[derive(Default)]
#[repr(C)]
struct ExampleQuad {
    base: Background,
}
impl ExampleQuad {
    fn draw(&mut self, cx: &mut Cx, label: &str) {
        let turtle = self.base.begin_turtle(
            cx,
            Layout {
                walk: Walk { width: Width::Compute, height: Height::Compute, margin: Margin::all(1.0) },
                padding: Padding { l: 16.0, t: 12.0, r: 16.0, b: 12.0 },
                ..Layout::default()
            },
            vec4(0.8, 0.2, 0.4, 1.),
        );
        TextIns::draw_walk(cx, label, &TextInsProps::DEFAULT);
        self.base.end_turtle(cx, turtle);
    }
}

pub struct LayoutExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    quad: ExampleQuad,
    token_input: TextInput,
}

impl LayoutExampleApp {
    pub fn new(_cx: &mut Cx) -> Self {
        Self {
            window: Window { create_inner_size: Some(Vec2 { x: 800., y: 600. }), ..Window::default() },
            pass: Pass::default(),
            quad: ExampleQuad::default(),
            main_view: View::default(),
            token_input: TextInput::new(TextInputOptions {
                empty_message: "Enter text".to_string(),
                ..TextInputOptions::default()
            }),
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        // match event {
        //     _ => (),
        // }
        self.token_input.handle(cx, event);
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("500"));
        self.main_view.begin_view(cx, Layout { direction: Direction::Down, ..Layout::default() });

        let top = cx.begin_turtle(Layout { walk: Walk::wh(Width::Fill, Height::Fix(27.)), ..Layout::default() });
        cx.end_turtle(top);

        // This is the example of applying various alignment techniques
        {
            // First we cut the row with quads being on both side (left / right) and the middle one spanning the remaining
            let row = cx.begin_row_turtle();
            {
                self.quad.draw(cx, "Row 1");
                self.quad.draw(cx, "Row 2");
                self.quad.draw(cx, "3");
                self.quad.draw(cx, "4");
            }
            {
                cx.begin_right_align();
                self.quad.draw(cx, "Row 5");
                self.quad.draw(cx, "Row 6");
                cx.end_right_align();
            }
            {
                cx.begin_center_x_align();
                self.quad.draw(cx, "Row mid");
                cx.end_center_x_align();
            }
            cx.end_turtle(row);
        }
        {
            // Cut fixed height row
            let row = cx.begin_turtle(Layout {
                direction: Direction::Right,
                walk: Walk { width: Width::Fill, height: Height::Fix(80.), margin: Margin::ZERO },
                ..Layout::default()
            });

            self.quad.draw(cx, "Fixed Row Top");
            {
                let column = cx.begin_column_turtle();
                cx.begin_center_y_align();
                self.quad.draw(cx, "Fixed Row Center");
                cx.end_center_y_align();
                cx.end_turtle(column);
            }
            {
                let column = cx.begin_column_turtle();
                cx.begin_bottom_align();
                self.quad.draw(cx, "Fixed Row Bottom");
                cx.end_bottom_align();
                cx.end_turtle(column);
            }
            cx.end_turtle(row);
        }
        {
            // Cut the column aligned on the right
            let row = cx.begin_turtle(Layout {
                direction: Direction::Right,
                walk: Walk { width: Width::Fill, height: Height::Fill, margin: Margin::ZERO },
                ..Layout::default()
            });
            {
                cx.begin_right_align();
                let column = cx.begin_column_turtle();
                {
                    self.quad.draw(cx, "Col 1");
                    self.quad.draw(cx, "some very long text");
                }
                {
                    cx.begin_center_y_align();
                    self.quad.draw(cx, "Col mid");
                    cx.end_center_y_align();
                }
                cx.end_turtle(column);
                cx.end_right_align();
            }
            {
                // Finally the remaining block has the quad centered by both x and y axis
                cx.begin_center_x_and_y_align();
                {
                    self.quad.draw(cx, "Mid 1");
                    self.token_input.draw(cx);
                    self.quad.draw(cx, "Mid 2");
                }
                cx.end_center_x_and_y_align();
            }
            cx.end_turtle(row);
        }

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(LayoutExampleApp);
