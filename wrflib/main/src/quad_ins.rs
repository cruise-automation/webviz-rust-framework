// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! Drawing rectangles; by far the most commonly used `Draw*` struct.

use crate::*;

/// [`QuadIns`] is the basis for most draw structs.This renders a rectangle.
/// There are some default shaders available at [`QuadIns::SHADER`].
///
/// Example usage with your own struct:
///
/// ```
/// struct MyStruct {
///   pub base: QuadIns,
///   pub field1: f32,
///   pub field2: f32,
/// }
/// ```
///
/// And render using:
///
/// ```
/// let s = DrawMyStruct {
///   base: QuadIns::from_rect(rect),
///   field1: 0.0,
///   field2: 0.0,
/// };
/// cx.add_instances(&SHADER, &[s]);
/// ```
#[derive(Clone, Default)]
#[repr(C)]
pub struct QuadIns {
    /// The top-left corner position of the quad, in absolute coordinates.
    pub rect_pos: Vec2,
    /// The size of the quad.
    pub rect_size: Vec2,
    /// Z-index.
    pub draw_depth: f32,
}

impl QuadIns {
    pub fn from_rect(rect: Rect) -> Self {
        debug_assert!(!rect.size.x.is_nan());
        debug_assert!(!rect.size.y.is_nan());
        Self { rect_pos: rect.pos, rect_size: rect.size, ..Default::default() }
    }

    pub fn with_draw_depth(mut self, draw_depth: f32) -> Self {
        self.draw_depth = draw_depth;
        self
    }

    pub fn rect(&self) -> Rect {
        Rect { pos: self.rect_pos, size: self.rect_size }
    }

    pub fn build_geom() -> Geometry {
        let vertex_attributes = vec![vec2(0., 0.), vec2(1., 0.), vec2(1., 1.), vec2(0., 1.)];
        let indices = vec![[0, 1, 2], [2, 3, 0]];
        Geometry::new(vertex_attributes, indices)
    }

    /// Common [`Shader`] code for using [`QuadIns`].
    pub const SHADER: CodeFragment = code_fragment!(
        r#"
        instance rect_pos: vec2;
        instance rect_size: vec2;
        instance draw_depth: float;
        geometry geom: vec2;
        varying pos: vec2;

        fn scroll() -> vec2 {
            return draw_scroll;
        }

        fn vertex() -> vec4 {
            let scr = scroll();

            let clipped: vec2 = clamp(
                geom * rect_size + rect_pos - scr,
                draw_clip.xy,
                draw_clip.zw
            );
            pos = (clipped + scr - rect_pos) / rect_size;
            // only pass the clipped position forward
            return camera_projection * (camera_view * vec4(
                clipped.x,
                clipped.y,
                draw_depth + draw_zbias,
                1.
            ));
        }
    "#
    );
}
