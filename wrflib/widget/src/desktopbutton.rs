// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::buttonlogic::*;
use wrflib::*;

pub enum DesktopButtonType {
    WindowsMin,
    WindowsMax,
    WindowsMaxToggled,
    WindowsClose,
    XRMode,
    Fullscreen,
}

impl DesktopButtonType {
    fn shader_float(&self) -> f32 {
        match self {
            DesktopButtonType::WindowsMin => 1.,
            DesktopButtonType::WindowsMax => 2.,
            DesktopButtonType::WindowsMaxToggled => 3.,
            DesktopButtonType::WindowsClose => 4.,
            DesktopButtonType::XRMode => 5.,
            DesktopButtonType::Fullscreen => 6.,
        }
    }
}

#[derive(Clone, Default)]
#[repr(C)]
struct DesktopButtonIns {
    base: QuadIns,
    hover: f32,
    down: f32,
    button_type: f32,
}

static SHADER: Shader = Cx::define_shader(
    Some(GEOM_QUAD2D),
    &[Cx::STD_SHADER, QuadIns::SHADER],
    code_fragment!(
        r#"
        instance hover: float;
        instance down: float;
        instance button_type: float;

        fn pixel() -> vec4 {
            let df = Df::viewport(pos * rect_size);
            df.aa *= 3.0;
            let sz = 4.5;
            let c = rect_size * vec2(0.5, 0.5);
            // WindowsMin
            if abs(button_type - 1.) < 0.1 {
                df.clear(mix(#3, mix(#6, #9, down), hover));
                df.move_to(c.x - sz, c.y);
                df.line_to(c.x + sz, c.y);
                df.stroke(#f, 0.5 + 0.5 * dpi_dilate);
                return df.result;
            }
            // WindowsMax
            if abs(button_type - 2.) < 0.1 {
                df.clear(mix(#3, mix(#6, #9, down), hover));
                df.rect(c.x - sz, c.y - sz, 2. * sz, 2. * sz);
                df.stroke(#f, 0.5 + 0.5 * dpi_dilate);
                return df.result;
            }
            // WindowsMaxToggled
            if abs(button_type - 3.) < 0.1 {
                let clear = mix(#3, mix(#6, #9, down), hover);
                df.clear(clear);
                let sz = 3.5;
                df.rect(c.x - sz + 1., c.y - sz - 1., 2. * sz, 2. * sz);
                df.stroke(#f, 0.5 + 0.5 * dpi_dilate);
                df.rect(c.x - sz - 1., c.y - sz + 1., 2. * sz, 2. * sz);
                df.fill_keep(clear);
                df.stroke(#f, 0.5 + 0.5 * dpi_dilate);

                return df.result;
            }
            // WindowsClose
            if abs(button_type - 4.) < 0.1 {
                df.clear(mix(#3, mix(#e00, #c00, down), hover));
                df.move_to(c.x - sz, c.y - sz);
                df.line_to(c.x + sz, c.y + sz);
                df.move_to(c.x - sz, c.y + sz);
                df.line_to(c.x + sz, c.y - sz);
                df.stroke(#f, 0.5 + 0.5 * dpi_dilate);
                return df.result;
            }
            // VRMode
            if abs(button_type - 5.) < 0.1 {
                df.clear(mix(#3, mix(#0aa, #077, down), hover));
                let w = 12.;
                let h = 8.;
                df.box(c.x - w, c.y - h, 2. * w, 2. * h, 2.);
                // subtract 2 eyes
                df.circle(c.x - 5.5, c.y, 3.5);
                df.subtract();
                df.circle(c.x + 5.5, c.y, 3.5);
                df.subtract();
                df.circle(c.x, c.y + h - 0.75, 2.5);
                df.subtract();
                df.fill(#8);

                return df.result;
            }
            // Fullscreen
            if abs(button_type - 6.) < 0.1 {
                sz = 8.;
                df.clear(mix(#3, mix(#6, #9, down), hover));
                df.rect(c.x - sz, c.y - sz, 2. * sz, 2. * sz);
                df.rect(c.x - sz + 1.5, c.y - sz + 1.5, 2. * (sz - 1.5), 2. * (sz - 1.5));
                df.subtract();
                df.rect(c.x - sz + 4., c.y - sz - 2., 2. * (sz - 4.), 2. * (sz + 2.));
                df.subtract();
                df.rect(c.x - sz - 2., c.y - sz + 4., 2. * (sz + 2.), 2. * (sz - 4.));
                df.subtract();
                df.fill(#f); //, 0.5 + 0.5 * dpi_dilate);

                return df.result;
            }

            return #f00;
        }"#
    ),
);

#[derive(Default)]
pub struct DesktopButton {
    component_base: ComponentBase,
    bg_area: Area,
    animator: Animator,
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // DesktopButtonIns::hover
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // DesktopButtonIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_HOVER: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // DesktopButtonIns::hover
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // DesktopButtonIns::down
        Track::Float { key_frames: &[(0.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // DesktopButtonIns::hover
        Track::Float { key_frames: &[(0.0, 1.0)], ease: Ease::DEFAULT },
        // DesktopButtonIns::down
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

impl DesktopButton {
    fn animate(&mut self, cx: &mut Cx) {
        let bg = self.bg_area.get_first_mut::<DesktopButtonIns>(cx);
        bg.hover = self.animator.get_float(0);
        bg.down = self.animator.get_float(1);
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ButtonEvent {
        if self.animator.handle(cx, event) {
            self.animate(cx);
        }
        let animator = &mut self.animator;
        let hit_event = event.hits(cx, &self.component_base, HitOpt::default());
        handle_button_logic(cx, hit_event, |cx, logic_event| match logic_event {
            ButtonLogicEvent::Down => animator.play_anim(cx, ANIM_DOWN),
            ButtonLogicEvent::Default => animator.play_anim(cx, ANIM_DEFAULT),
            ButtonLogicEvent::Over => animator.play_anim(cx, ANIM_HOVER),
        })
    }

    pub fn draw(&mut self, cx: &mut Cx, ty: DesktopButtonType) {
        let (w, h) = match ty {
            DesktopButtonType::WindowsMin
            | DesktopButtonType::WindowsMax
            | DesktopButtonType::WindowsMaxToggled
            | DesktopButtonType::WindowsClose => (46., 29.),
            DesktopButtonType::XRMode => (50., 36.),
            DesktopButtonType::Fullscreen => (50., 36.),
        };

        let rect = cx.walk_turtle(Walk::wh(Width::Fix(w), Height::Fix(h)));

        self.bg_area = cx.add_instances(
            &SHADER,
            &[DesktopButtonIns { base: QuadIns::from_rect(rect), button_type: ty.shader_float(), ..Default::default() }],
        );

        self.component_base.register_component_area(cx, self.bg_area);

        self.animator.draw(cx, ANIM_DEFAULT);
        self.animate(cx);
    }
}
