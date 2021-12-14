// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! Collection of standard [`Shader`] functions.

use crate::cx::*;

impl Cx {
    /// Collection of standard [`Shader`] functions.
    // Based on https://www.shadertoy.com/view/lslXW8
    pub const STD_SHADER: CodeFragment = code_fragment!(
        r#"
        // See [`PassUniforms`] for documentation on these fields.
        uniform camera_projection: mat4 in pass;
        uniform camera_view: mat4 in pass;
        uniform inv_camera_rot: mat4 in pass;
        uniform dpi_factor: float in pass;
        uniform dpi_dilate: float in pass;

        // See [`DrawUniforms`] for documentation on these fields.
        uniform draw_clip: vec4 in draw;
        uniform draw_scroll: vec2 in draw;
        uniform draw_local_scroll: vec2 in draw;
        uniform draw_zbias: float in draw;

        const PI: float = 3.141592653589793;
        const E: float = 2.718281828459045;
        const LN2: float = 0.6931471805599453;
        const LN10: float = 2.302585092994046;
        const LOG2E: float = 1.4426950408889634;
        const LOG10E: float = 0.4342944819032518;
        const SQRT1_2: float = 0.70710678118654757;
        const TORAD: float = 0.017453292519943295;
        const GOLDEN: float = 1.618033988749895;

        // The current distance field
        struct Df {
            pos: vec2,
            result: vec4,
            last_pos: vec2,
            start_pos: vec2,
            shape: float,
            clip: float,
            has_clip: float,
            old_shape: float,
            blur: float,
            aa: float,
            scale: float,
            field: float
        }

        impl Math{
            fn rotate_2d(v: vec2, a: float)->vec2 {
                let ca = cos(a);
                let sa = sin(a);
                return vec2(v.x * ca - v.y * sa, v.x * sa + v.y * ca);
            }
        }

        impl Pal {
            fn iq(t: float, a: vec3, b: vec3, c: vec3, d: vec3) -> vec3 {
                return a + b * cos(6.28318 * (c * t + d));
            }

            fn iq0(t: float) -> vec3 {
                return mix(vec3(0., 0., 0.), vec3(1., 1., 1.), cos(t * PI) * 0.5 + 0.5);
            }

            fn iq1(t: float) -> vec3 {
                return Pal::iq(t, vec3(0.5, 0.5, 0.5), vec3(0.5, 0.5, 0.5), vec3(1., 1., 1.), vec3(0., 0.33, 0.67));
            }

            fn iq2(t: float) -> vec3 {
                return Pal::iq(t, vec3(0.5, 0.5, 0.5), vec3(0.5, 0.5, 0.5), vec3(1., 1., 1.), vec3(0., 0.1, 0.2));
            }

            fn iq3(t: float) -> vec3 {
                return Pal::iq(t, vec3(0.5, 0.5, 0.5), vec3(0.5, 0.5, 0.5), vec3(1., 1., 1.), vec3(0.3, 0.2, 0.2));
            }

            fn iq4(t: float) -> vec3 {
                return Pal::iq(t, vec3(0.5, 0.5, 0.5), vec3(0.5, 0.5, 0.5), vec3(1., 1., 0.5), vec3(0.8, 0.9, 0.3));
            }

            fn iq5(t: float) -> vec3 {
                return Pal::iq(t, vec3(0.5, 0.5, 0.5), vec3(0.5, 0.5, 0.5), vec3(1., 0.7, 0.4), vec3(0, 0.15, 0.20));
            }

            fn iq6(t: float) -> vec3 {
                return Pal::iq(t, vec3(0.5, 0.5, 0.5), vec3(0.5, 0.5, 0.5), vec3(2., 1.0, 0.), vec3(0.5, 0.2, 0.25));
            }

            fn iq7(t: float) -> vec3 {
                return Pal::iq(t, vec3(0.8, 0.5, 0.4), vec3(0.2, 0.4, 0.2), vec3(2., 1.0, 1.0), vec3(0., 0.25, 0.25));
            }

            //http://gamedev.stackexchange.com/questions/59797/glsl-shader-change-hue-saturation-brightness
            fn hsv2rgb(c: vec4) -> vec4 {
                let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
                let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
                return vec4(c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y), c.w);
            }

            fn rgb2hsv(c: vec4) -> vec4 {
                let K: vec4 = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
                let p: vec4 = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
                let q: vec4 = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

                let d: float = q.x - min(q.w, q.y);
                let e: float = 1.0e-10;
                return vec4(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x, c.w);
            }
        }

        impl Df {
            // Creates a distance field with the current position
            fn viewport(pos: vec2) -> Df {
                let df: Df;
                df.pos = pos;
                df.result = vec4(0.);
                df.last_pos = vec2(0.);
                df.start_pos = vec2(0.);
                df.shape = 1e+20;
                df.clip = -1e+20;
                df.has_clip = 0.0;
                df.old_shape = 1e+20;
                df.blur = 0.00001;
                df.aa = Df::antialias(pos);
                df.scale = 1.0;
                df.field = 0.0;
                return df;
            }

            // Creates a distance field with the current position, matching pixel scale
            fn viewport_px(pos: vec2) -> Df {
                return Df::viewport(pos * dpi_factor);
            }

            // Adds a new field value to the current distance field
            fn add_field(inout self, field: float) {
                self.field = field / self.scale;
                self.old_shape = self.shape;
                self.shape = min(self.field, self.shape);
            }

            // Adds a clip mask to the current distance field
            fn add_clip(inout self, d: float) {
                d = d / self.scale;
                self.clip = max(self.clip, d);
                self.has_clip = 1.;
            }

            fn antialias(p: vec2) -> float {
                return 1.0 / length(vec2(length(dFdx(p)), length(dFdy(p))));
            }

            // Performs a translation with offset (x, y)
            fn translate(inout self, x: float, y: float) -> vec2 {
                self.pos -= vec2(x, y);
                return self.pos;
            }

            // Performs a rotation by `a` radians and pivot (x, y)
            fn rotate(inout self, a: float, x: float, y: float) {
                let ca = cos(-a);
                let sa = sin(-a);
                let p = self.pos - vec2(x, y);
                self.pos = vec2(p.x * ca - p.y * sa, p.x * sa + p.y * ca) + vec2(x, y);
            }

            // Performs uniform scaling by `f` factor and pivot (x, y)
            fn scale(inout self, f: float, x: float, y: float) {
                self.scale *= f;
                self.pos = (self.pos - vec2(x, y)) * f + vec2(x, y);
            }

            // TODO(hernan): Add documentation
            fn clear(inout self, color: vec4) {
                self.result = vec4(color.rgb * color.a + self.result.rgb * (1.0 - color.a), color.a);
            }

            // Calculate antialising blur
            fn calc_blur(inout self, w: float) -> float {
                let wa = clamp(-w * self.aa, 0.0, 1.0);
                let wb = 1.0;
                if self.blur > 0.001 {
                    wb = clamp(-w / self.blur, 0.0, 1.0);
                }
                return wa * wb;
            }

            // Clears path in current distance field.
            fn new_path(inout self) -> vec4 {
                self.old_shape = self.shape = 1e+20;
                self.clip = -1e+20;
                self.has_clip = 0.;
                return self.result;
            }

            // Writes a color to the distance field, using premultiplied alpha
            fn write_color(inout self, src: vec4, w: float) -> vec4{
                let src_a = src.a * w;
                self.result = src * src_a + (1. - src_a) * self.result;
                return self.result;
            }

            // Fills and preserves the current path in the distance field, allowing further operations on it.
            fn fill_keep(inout self, color: vec4) -> vec4 {
                let f = self.calc_blur(self.shape);
                self.write_color(color, f);
                if self.has_clip > 0. {
                    self.write_color(color, self.calc_blur(self.clip));
                }
                return self.result;
            }

            // Fills the current path in the distance field and clears it.
            fn fill(inout self, color: vec4) -> vec4 {
                self.fill_keep(color);
                return self.new_path();
            }

            // Strokes and preserves the current path in the distance field, allowing further operations on it.
            fn stroke_keep(inout self, color: vec4, width: float) -> vec4 {
                let f = self.calc_blur(abs(self.shape) - width / self.scale);
                return self.write_color(color, f);
            }

            // Strokes the current path in the distance field and clears it.
            fn stroke(inout self, color: vec4, width: float) -> vec4 {
                self.stroke_keep(color, width);
                return self.new_path();
            }

            fn glow_keep(inout self, color: vec4, width: float) -> vec4 {
                let f = self.calc_blur(abs(self.shape) - width / self.scale);
                let source = vec4(color.rgb * color.a, color.a);
                let dest = self.result;
                self.result = vec4(source.rgb * f, 0.) + dest;
                return self.result;
            }

            fn glow(inout self, color: vec4, width: float) -> vec4 {
                self.glow_keep(color, width);
                self.old_shape = self.shape = 1e+20;
                self.clip = -1e+20;
                self.has_clip = 0.;
                return self.result;
            }

            fn union(inout self) {
                self.old_shape = self.shape = min(self.field, self.old_shape);
            }

            fn intersect(inout self) {
                self.old_shape = self.shape = max(self.field, self.old_shape);
            }

            fn subtract(inout self) {
                self.old_shape = self.shape = max(-self.field, self.old_shape);
            }

            fn gloop(inout self, k: float) {
                let h = clamp(0.5 + 0.5 * (self.old_shape - self.field) / k, 0.0, 1.0);
                self.old_shape = self.shape = mix(self.old_shape, self.field, h) - k * h * (1.0 - h);
            }

            fn blend(inout self, k: float) {
                self.old_shape = self.shape = mix(self.old_shape, self.field, k);
            }

            // Renders a circle at point (x, y) with radius r
            fn circle(inout self, x: float, y: float, r: float) {
                let c = self.pos - vec2(x, y);
                self.add_field(length(c) - r);
            }

            fn arc(inout self, x: float, y: float, r: float, angle_start: float, angle_end: float) {
                let c = self.pos - vec2(x, y);
                let angle = mod(atan(c.x, -c.y) + 2.*PI, 2.*PI);
                let d = max( angle_start - angle, angle - angle_end );
                let len = max(length(c) * d, length(c) - r);
                self.add_field(len / self.scale);
            }

            // A box with rounded corners.
            // Use `r` to indicate the corner radius. If the radius is less than 1,
            // we just render a basic rectangle. If the value of radius is bigger than
            // min(w, h), the result will be a circle.
            fn box(inout self, x: float, y: float, w: float, h: float, r: float) {
                let s = 0.5 * vec2(w, h);
                let o = vec2(x, y) + s;
                r = min(r, min(w, h));
                s -= r;
                let d = abs(o - self.pos) - s;
                let dmin = min(d, 0.);
                let dmax = max(d, 0.);
                let df = max(dmin.x, dmin.y) + length(dmax);
                self.add_field(df - r);
            }

            fn rect(inout self, x: float, y: float, w: float, h: float) {
                self.box(x, y, w, h, 0.);
            }

            fn triangle(inout self, p0: vec2, p1: vec2, p2: vec2) {
                let e0 = p1 - p0;
                let e1 = p2 - p1;
                let e2 = p0-p2;

                let v0 = self.pos - p0;
                let v1 = self.pos - p1;
                let v2 = self.pos - p2;

                let pq0 = v0 - e0 * clamp(dot(v0, e0) / dot(e0, e0), 0.0, 1.0);
                let pq1 = v1 - e1 * clamp(dot(v1, e1) / dot(e1, e1), 0.0, 1.0);
                let pq2 = v2 - e2 * clamp(dot(v2, e2) / dot(e2, e2), 0.0, 1.0);

                let s = sign(e0.x * e2.y - e0.y * e2.x);
                let d = min(min(vec2(dot(pq0, pq0), s*(v0.x * e0.y - v0.y * e0.x)),
                        vec2(dot(pq1, pq1), s * (v1.x * e1.y - v1.y * e1.x))),
                        vec2(dot(pq2, pq2), s * (v2.x * e2.y - v2.y * e2.x)));

                self.add_field(-sqrt(d.x) * sign(d.y));
            }

            fn hexagon(inout self, x: float, y: float, r: float) {
                let dx = abs(x - self.pos.x) * 1.15;
                let dy = abs(y - self.pos.y);
                self.add_field(max(dy + cos(60.0 * TORAD) * dx - r, dx - r));
            }

            fn move_to(inout self, x: float, y: float) {
                self.last_pos =
                self.start_pos = vec2(x, y);
            }

            fn line_to(inout self, x: float, y: float) {
                let p = vec2(x, y);

                let pa = self.pos - self.last_pos;
                let ba = p - self.last_pos;
                let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
                let s = sign(pa.x * ba.y - pa.y * ba.x);
                self.field = length(pa - ba * h) / self.scale;
                self.old_shape = self.shape;
                self.shape = min(self.shape, self.field);
                self.clip = max(self.clip, self.field * s);
                self.has_clip = 1.0;
                self.last_pos = p;
            }

            fn close_path(inout self) {
                self.line_to(self.start_pos.x, self.start_pos.y);
            }
        }
    "#
    );
}
