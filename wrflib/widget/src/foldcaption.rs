// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::buttonlogic::*;
use wrflib::*;

#[derive(Clone)]
pub enum FoldOpenState {
    Open,
    Opening(f32),
    Closed,
    Closing(f32),
}

impl Default for FoldOpenState {
    fn default() -> Self {
        FoldOpenState::Open
    }
}

impl FoldOpenState {
    fn get_value(&self) -> f32 {
        match self {
            FoldOpenState::Opening(fac) => 1.0 - *fac,
            FoldOpenState::Closing(fac) => *fac,
            FoldOpenState::Open => 1.0,
            FoldOpenState::Closed => 0.0,
        }
    }
    pub fn is_open(&self) -> bool {
        match self {
            FoldOpenState::Opening(_) => true,
            FoldOpenState::Closing(_) => false,
            FoldOpenState::Open => true,
            FoldOpenState::Closed => false,
        }
    }
    pub fn toggle(&mut self) {
        *self = match self {
            FoldOpenState::Opening(fac) => FoldOpenState::Closing(1.0 - *fac),
            FoldOpenState::Closing(fac) => FoldOpenState::Opening(1.0 - *fac),
            FoldOpenState::Open => FoldOpenState::Closing(1.0),
            FoldOpenState::Closed => FoldOpenState::Opening(1.0),
        };
    }
    pub fn do_open(&mut self) {
        *self = match self {
            FoldOpenState::Opening(fac) => FoldOpenState::Opening(*fac),
            FoldOpenState::Closing(fac) => FoldOpenState::Opening(1.0 - *fac),
            FoldOpenState::Open => FoldOpenState::Open,
            FoldOpenState::Closed => FoldOpenState::Opening(1.0),
        };
    }
    pub fn do_close(&mut self) {
        *self = match self {
            FoldOpenState::Opening(fac) => FoldOpenState::Closing(1.0 - *fac),
            FoldOpenState::Closing(fac) => FoldOpenState::Closing(*fac),
            FoldOpenState::Open => FoldOpenState::Closing(1.0),
            FoldOpenState::Closed => FoldOpenState::Closed,
        };
    }
    pub fn do_time_step(&mut self, mul: f32) -> bool {
        let mut redraw = false;
        *self = match self {
            FoldOpenState::Opening(fac) => {
                redraw = true;
                if *fac < 0.001 {
                    FoldOpenState::Open
                } else {
                    FoldOpenState::Opening(*fac * mul)
                }
            }
            FoldOpenState::Closing(fac) => {
                redraw = true;
                if *fac < 0.001 {
                    FoldOpenState::Closed
                } else {
                    FoldOpenState::Closing(*fac * mul)
                }
            }
            FoldOpenState::Open => FoldOpenState::Open,
            FoldOpenState::Closed => FoldOpenState::Closed,
        };
        redraw
    }
}

#[derive(Clone, Default)]
#[repr(C)]
struct FoldCaptionIns {
    base: QuadIns,
    hover: f32,
    down: f32,
    open: f32,
}

static SHADER: Shader = Cx::define_shader(
    Some(GEOM_QUAD2D),
    &[Cx::STD_SHADER, QuadIns::SHADER],
    code_fragment!(
        r#"
    instance hover: float;
    instance down: float;
    instance open: float;

    const shadow: float = 3.0;
    const border_radius: float = 2.5;

    fn pixel() -> vec4 {
        let sz = 3.;
        let c = vec2(5.0,0.5*rect_size.y);
        let df = Df::viewport(pos * rect_size);
        df.clear(#2);
        // we have 3 points, and need to rotate around its center
        df.rotate(open*0.5*PI+0.5*PI, c.x, c.y);
        df.move_to(c.x - sz, c.y + sz);
        df.line_to(c.x, c.y - sz);
        df.line_to(c.x + sz, c.y + sz);
        df.close_path();
        df.fill(mix(#a,#f,hover));

        return df.result;
    }"#
    ),
);

#[derive(Default)]
pub struct FoldCaption {
    component_base: ComponentBase,
    bg_area: Area,
    text_area: Area,
    animator: Animator,
    open_state: FoldOpenState,
    turtle: Option<Turtle>,
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.1,
    tracks: &[
        // FoldCaptionIns::hover
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // FoldCaptionIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // TextIns::color
        Track::Vec4 { key_frames: &[(1.0, vec4(0.6, 0.6, 0.6, 1.0))], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_OVER: Anim = Anim {
    duration: 0.1,
    tracks: &[
        // FoldCaptionIns::hover
        Track::Float { key_frames: &[(0.0, 1.0), (1.0, 1.0)], ease: Ease::DEFAULT },
        // FoldCaptionIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // TextIns::color
        Track::Vec4 { key_frames: &[(0.0, Vec4::all(1.))], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // FoldCaptionIns::hover
        Track::Float { key_frames: &[(0.0, 1.0), (1.0, 1.0)], ease: Ease::DEFAULT },
        // FoldCaptionIns::down
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // TextIns::color
        Track::Vec4 { key_frames: &[(0.0, vec4(0.8, 0.8, 0.8, 1.0))], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

impl FoldCaption {
    fn animate(&mut self, cx: &mut Cx) {
        let bg = self.bg_area.get_first_mut::<FoldCaptionIns>(cx);
        bg.hover = self.animator.get_float(0);
        bg.down = self.animator.get_float(1);
        TextIns::set_color(cx, self.text_area, self.animator.get_vec4(2));
    }

    pub fn handle_fold_caption(&mut self, cx: &mut Cx, event: &mut Event) -> ButtonEvent {
        if self.animator.handle(cx, event) {
            self.animate(cx);
        }

        let animator = &mut self.animator;
        let open_state = &mut self.open_state;
        let hit_event = event.hits(cx, &self.component_base, HitOpt::default());
        handle_button_logic(cx, hit_event, |cx, logic_event| match logic_event {
            ButtonLogicEvent::Down => {
                // lets toggle our anim state
                open_state.toggle();
                cx.request_draw();
                animator.play_anim(cx, ANIM_DOWN);
            }
            ButtonLogicEvent::Default => animator.play_anim(cx, ANIM_DEFAULT),
            ButtonLogicEvent::Over => animator.play_anim(cx, ANIM_OVER),
        })
    }

    pub fn begin_fold_caption(&mut self, cx: &mut Cx) -> f32 {
        let open_value = self.open_state.get_value();

        self.bg_area = cx.add_instances(&SHADER, &[FoldCaptionIns { open: open_value, ..Default::default() }]);

        self.turtle = Some(cx.begin_turtle(Layout {
            walk: Walk { width: Width::Fill, height: Height::Compute, margin: Margin::all(1.0) },
            padding: Padding { l: 14.0, t: 8.0, r: 14.0, b: 8.0 },
            ..Layout::default()
        }));

        if self.open_state.do_time_step(0.6) {
            cx.request_draw();
        }

        open_value
    }

    pub fn end_fold_caption(&mut self, cx: &mut Cx, label: &str) {
        cx.reset_turtle_pos();

        let text_turtle = cx.begin_turtle(Layout {
            walk: Walk { width: Width::Fill, height: Height::Compute, ..Walk::default() },
            ..Layout::default()
        });
        // TODO(Dmitry): using only the right alignment without ecnlosing turtle seems to be reducing the width
        // of parent turtle and returning shorter rect as a result causing not fully covered background
        cx.begin_right_align();
        let draw_str_props = TextInsProps { wrapping: Wrapping::Ellipsis(cx.get_width_left() - 10.), ..TextInsProps::DEFAULT };
        TextIns::draw_walk(cx, label, &draw_str_props);
        cx.end_right_align();
        cx.end_turtle(text_turtle);

        let rect = cx.end_turtle(self.turtle.take().unwrap());

        let bg = self.bg_area.get_first_mut::<FoldCaptionIns>(cx);
        bg.base = QuadIns::from_rect(rect);

        self.component_base.register_component_area(cx, self.bg_area);

        self.animator.draw(cx, ANIM_DEFAULT);
        self.animate(cx);
    }
}
