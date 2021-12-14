// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! Convenient way of drawing colored rectangles, built on top of [`QuadIns`].

use wrflib::*;

#[derive(Clone, Default)]
#[repr(C)]
struct BackgroundIns {
    quad: QuadIns,
    color: Vec4,
    radius: f32,
}

static SHADER: Shader = Cx::define_shader(
    Some(GEOM_QUAD2D),
    &[Cx::STD_SHADER, QuadIns::SHADER],
    code_fragment!(
        r#"
        instance color: vec4;
        instance radius: float;
        fn pixel() -> vec4 {
            // TODO(JP): Giant hack! We should just be able to call df.box with radius=0
            // and then df.fill, but df.box with radius=0 seems totally broken, and even
            // using df.rect with df.fill seems to leave a gap around the border..
            if radius < 0.001 {
                return vec4(color.rgb*color.a, color.a);
            }

            let df = Df::viewport(pos * rect_size);
            df.box(0., 0., rect_size.x, rect_size.y, radius);
            return df.fill(color);
        }"#
    ),
);

#[derive(Default)]
pub struct Background {
    area: Area,
    radius: f32,
    draw_depth: f32,
}

impl Background {
    pub fn set_color(&mut self, cx: &mut Cx, color: Vec4) {
        let bg = self.area.get_first_mut::<BackgroundIns>(cx);
        bg.color = color;
    }
    pub fn with_draw_depth(self, draw_depth: f32) -> Self {
        Self { draw_depth, ..self }
    }
    pub fn with_radius(self, radius: f32) -> Self {
        Self { radius, ..self }
    }

    /// Calls [`Self::draw`] without having to pass in a [`Rect`] immediately. We will overwrite
    /// the coordinates in the shader directly in [`Background::end_turtle`].
    ////
    /// This is useful for if you need to draw a quad in the background, since in that case you have
    /// to draw the quad first before drawing the content (otherwise it would sit on top of the
    /// content!), but you might not know the dimensions yet. In [`Background::end_turtle`] we
    /// get the dimensions of the content from [`Cx::end_turtle`] and set this directly using
    /// [`Area::get_first_mut`].
    #[must_use]
    pub fn begin_turtle(&mut self, cx: &mut Cx, layout: Layout, color: Vec4) -> Turtle {
        self.draw(cx, Rect::default(), color);
        cx.begin_turtle(layout)
    }

    /// See [`Background::begin_turtle`].
    pub fn end_turtle(&mut self, cx: &mut Cx, turtle: Turtle) {
        let rect = cx.end_turtle(turtle);
        let bg = self.area.get_first_mut::<BackgroundIns>(cx);
        bg.quad.rect_pos = rect.pos;
        bg.quad.rect_size = rect.size;
    }

    /// Get the [`Area`].
    pub fn area(&self) -> Area {
        self.area
    }

    /// Manually set the [`Area`].
    pub fn set_area(&mut self, area: Area) {
        self.area = area;
    }

    /// Draw the background.
    pub fn draw(&mut self, cx: &mut Cx, rect: Rect, color: Vec4) {
        let data = BackgroundIns {
            quad: QuadIns { rect_pos: rect.pos, rect_size: rect.size, draw_depth: self.draw_depth },
            color,
            radius: self.radius,
        };
        self.area = cx.add_instances(&SHADER, &[data]);
    }

    /// Draw the background, but make it sticky with respect to scrolling. Not typically recommended.
    pub fn draw_with_scroll_sticky(&mut self, cx: &mut Cx, rect: Rect, color: Vec4) {
        let data = BackgroundIns {
            quad: QuadIns { rect_pos: rect.pos, rect_size: rect.size, draw_depth: self.draw_depth },
            color,
            radius: self.radius,
        };
        self.area = cx.add_instances_with_scroll_sticky(&SHADER, &[data], true, true);
    }
}
