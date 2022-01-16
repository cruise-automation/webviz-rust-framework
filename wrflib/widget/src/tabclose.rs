// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::buttonlogic::*;
use wrflib::*;

#[derive(Clone, Default)]
#[repr(C)]
struct TabCloseIns {
    base: QuadIns,
    color: Vec4,
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
            instance color: vec4;
            instance hover: float;
            instance down: float;

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);
                let hover_max: float = (hover * 0.4 + 0.3) * 0.5;
                let hover_min: float = 1. - hover_max;
                let c: vec2 = rect_size * 0.5;
                df.circle(c.x, c.y, 9.6);
                df.stroke_keep(#4000,1.);
                df.fill(mix(#3332,#555f,hover));
                df.rotate(down, c.x, c.y);
                df.move_to(c.x * hover_min, c.y * hover_min);
                df.line_to(c.x + c.x * hover_max, c.y + c.y * hover_max);
                df.move_to(c.x + c.x * hover_max, c.y * hover_min);
                df.line_to(c.x * hover_min, c.y + c.y * hover_max);
                return df.stroke(color, 1. + hover*0.2);
                //return df_fill(color);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub struct TabClose {
    component_base: ComponentBase,
    bg_area: Area,
    animator: Animator,
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // TabCloseIns::color
        Track::Vec4 { key_frames: &[(1.0, vec4(0.62, 0.62, 0.62, 1.))], ease: Ease::DEFAULT },
        // TabCloseIns::hover
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // TabCloseIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_OVER: Anim = Anim {
    duration: 0.1,
    tracks: &[
        // TabCloseIns::color
        Track::Vec4 { key_frames: &[(0.0, Vec4::all(1.))], ease: Ease::DEFAULT },
        // TabCloseIns::hover
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // TabCloseIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // TabCloseIns::color
        Track::Vec4 { key_frames: &[(0.0, Vec4::all(1.))], ease: Ease::DEFAULT },
        // TabCloseIns::hover
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // TabCloseIns::down
        Track::Float { key_frames: &[(0.0, 0.0), (1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

impl TabClose {
    fn animate(&mut self, cx: &mut Cx) {
        let bg = self.bg_area.get_first_mut::<TabCloseIns>(cx);
        bg.color = self.animator.get_vec4(0);
        bg.hover = self.animator.get_float(1);
        bg.down = self.animator.get_float(2);
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ButtonEvent {
        if self.animator.handle(cx, event) {
            self.animate(cx);
        }

        match event.hits(cx, &self.component_base, HitOpt { margin: Some(Margin::all(5.)), ..Default::default() }) {
            Event::FingerDown(_fe) => {
                self.animator.play_anim(cx, ANIM_DOWN);
                cx.set_down_mouse_cursor(MouseCursor::Hand);
                return ButtonEvent::Down;
            }
            Event::FingerHover(fe) => {
                cx.set_hover_mouse_cursor(MouseCursor::Hand);
                match fe.hover_state {
                    HoverState::In => {
                        if fe.any_down {
                            self.animator.play_anim(cx, ANIM_DOWN)
                        } else {
                            self.animator.play_anim(cx, ANIM_OVER)
                        }
                    }
                    HoverState::Out => self.animator.play_anim(cx, ANIM_DEFAULT),
                    _ => (),
                }
            }
            Event::FingerUp(fe) => {
                if fe.is_over {
                    if fe.input_type.has_hovers() {
                        self.animator.play_anim(cx, ANIM_OVER)
                    } else {
                        self.animator.play_anim(cx, ANIM_DEFAULT)
                    }
                    return ButtonEvent::Clicked;
                } else {
                    self.animator.play_anim(cx, ANIM_DEFAULT);
                    return ButtonEvent::Up;
                }
            }
            _ => (),
        };
        ButtonEvent::None
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        let rect = cx.walk_turtle(Walk::wh(Width::Fix(25.0), Height::Fix(25.0)));

        self.bg_area = cx
            .add_instances(&SHADER, &[TabCloseIns { base: QuadIns::from_rect(rect).with_draw_depth(1.3), ..Default::default() }]);

        self.component_base.register_component_area(cx, self.bg_area);

        self.animator.draw(cx, ANIM_DEFAULT);
        self.animate(cx);
    }
}
