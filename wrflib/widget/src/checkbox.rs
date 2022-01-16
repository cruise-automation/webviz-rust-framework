// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;

#[derive(Clone, Default)]
#[repr(C)]
struct CheckboxIns {
    base: QuadIns,
    checked: f32,
    loaded: f32,
    errored: f32,
    hover: f32,
    down: f32,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            uniform time: float;
            instance checked: float;
            instance loaded: float;
            instance errored: float;
            instance hover: float;
            instance down: float;
            const stroke_width: float = 1.1;
            const active_color: vec4 = #f0f0f0;
            const error_color: vec4 = #ff0000;
            const inactive_color: vec4 = #ccc;

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);

                df.rect(0., 0., rect_size.x, rect_size.y);
                df.fill(mix(vec4(0.,0.,0.,0.), vec4(1.,1.,1.,0.3), hover));

                let circle_size = 7. - stroke_width / 2. + down * 3.;
                df.circle(rect_size.y / 2., rect_size.y / 2., circle_size);
                if checked > 0. {
                    if errored > 0. {
                        df.stroke_keep(error_color, stroke_width);
                        df.fill(error_color);
                    }
                    else if loaded > 0. {
                        df.stroke_keep(active_color, stroke_width);
                        df.fill(active_color);
                    } else {
                        let t = time*2.;
                        let location = mod(t, 4.*PI);
                        let angle_start = 0.;
                        let angle_end = 0.;
                        if(location > 2.*PI) {
                            angle_start = 0.;
                            angle_end = mod(t, 2.*PI);
                        } else {
                            angle_start = mod(t, 2.*PI);
                            angle_end = 2.*PI;
                        }
                        df.stroke(inactive_color, stroke_width);
                        df.arc(rect_size.y / 2., rect_size.y / 2., circle_size, angle_start, angle_end);
                        df.stroke_keep(inactive_color, stroke_width);
                        df.fill(inactive_color);
                    }
                } else {
                    df.stroke(inactive_color, stroke_width);
                }

                return df.result;
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub struct Checkbox {
    component_base: ComponentBase,
    area: Area,
    animator: Animator,
}

#[derive(Clone, PartialEq)]
pub enum CheckboxEvent {
    None,
    Toggled,
}

const TEXT_STYLE: TextStyle = TextStyle { font_size: 10., ..TEXT_STYLE_NORMAL };

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.05,
    chain: true,
    tracks: &[
        // CheckboxIns::hover
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // CheckboxIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
};

const ANIM_HOVER: Anim = Anim {
    duration: 0.05,
    chain: true,
    tracks: &[
        // CheckboxIns::hover
        Track::Float { key_frames: &[(0.0, 1.0)], ease: Ease::DEFAULT },
        // CheckboxIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // CheckboxIns::hover
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // CheckboxIns::down
        Track::Float { key_frames: &[(0.5, 1.0), (1.0, 0.0)], ease: Ease::InQuad },
    ],
    ..Anim::DEFAULT
};

impl Checkbox {
    fn animator_animate(&mut self, cx: &mut Cx) {
        let checkbox = self.area.get_first_mut::<CheckboxIns>(cx);
        checkbox.hover = self.animator.get_float(0);
        checkbox.down = self.animator.get_float(1);
    }

    fn manual_animate(&mut self, cx: &mut Cx) {
        let checkbox = self.area.get_first::<CheckboxIns>(cx);
        if checkbox.checked > 0.0 && checkbox.loaded < 1.0 {
            self.area.write_user_uniforms(cx, cx.last_event_time as f32);
            cx.request_next_frame();
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> CheckboxEvent {
        if self.animator.handle(cx, event) {
            self.animator_animate(cx);
        }

        match event.hits(cx, &self.component_base, HitOpt::default()) {
            Event::FingerDown(_fe) => {
                let checkbox = self.area.get_first::<CheckboxIns>(cx);
                if checkbox.checked < 1.0 {
                    self.animator.play_anim(cx, ANIM_DOWN);
                }
                return CheckboxEvent::Toggled;
            }
            Event::FingerHover(fe) => {
                cx.set_hover_mouse_cursor(MouseCursor::Hand);
                match fe.hover_state {
                    HoverState::In => {
                        self.animator.play_anim(cx, ANIM_HOVER);
                    }
                    HoverState::Out => {
                        self.animator.play_anim(cx, ANIM_DEFAULT);
                    }
                    _ => (),
                }
            }
            _ => (),
        }

        if let Event::NextFrame = event {
            self.manual_animate(cx);
        }

        CheckboxEvent::None
    }

    pub fn draw(&mut self, cx: &mut Cx, checked: bool, loaded: bool, errored: bool, label: &str, fade_in_time: f64) {
        cx.begin_shader_group(&[&SHADER, &TEXT_INS_SHADER]);

        cx.begin_row(Width::Fill, Height::Fix(24.));
        cx.begin_padding_box(Padding::all(5.));

        if cx.last_event_time < fade_in_time {
            cx.request_draw();
        } else {
            let opacity = if cx.last_event_time < fade_in_time + 0.2 {
                cx.request_draw();
                (cx.last_event_time - fade_in_time) as f32 / 0.2
            } else {
                1.0
            };

            self.area = cx.add_instances(
                &SHADER,
                &[CheckboxIns {
                    base: QuadIns::from_rect(cx.get_box_rect()),
                    checked: checked as u8 as f32,
                    loaded: loaded as u8 as f32,
                    errored: errored as u8 as f32,
                    ..Default::default()
                }],
            );

            cx.walk_turtle(Walk::wh(Width::Fix(20.), Height::Fix(0.)));
            let draw_str_props = TextInsProps {
                wrapping: Wrapping::Ellipsis(cx.get_width_left() - 20.),
                color: if errored {
                    vec4(0.94, 0., 0., opacity)
                } else if checked {
                    vec4(0.94, 0.94, 0.94, opacity)
                } else {
                    vec4(0.8, 0.8, 0.8, opacity)
                },
                text_style: TEXT_STYLE,
                ..TextInsProps::DEFAULT
            };
            TextIns::draw_walk(cx, label, &draw_str_props);

            self.component_base.register_component_area(cx, self.area);

            self.animator.draw(cx, ANIM_DEFAULT);
            self.animator_animate(cx);

            self.manual_animate(cx);
        }
        cx.end_padding_box();
        cx.end_row();

        cx.end_shader_group();
    }
}
