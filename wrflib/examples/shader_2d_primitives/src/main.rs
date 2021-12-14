// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;
use wrflib_widget::*;

// TODO(hernan): Wrflib uses rounded corners for strokes. I guess this has to do with the SDF nature
// of the rendering functions, right?
// TODO(hernan): Arc is closed in Wrflib by default. Notice the 2nd row in both cases.
// TODO(hernan): Canvas has antialiasing for rendering. I use the `cx.translate(0.5, 0.5)` trick to
// make sure lines render at the center of the pixel, but there's still some antialiasing applied.
// TODO(hernan): When rendering paths in Wrflib (6th and 7th rows), the fill operation is different
// than in Canvas.
static MAIN_SHADER: Shader = Cx::define_shader(
    Some(GEOM_QUAD2D),
    &[Cx::STD_SHADER, QuadIns::SHADER],
    code_fragment!(
        r#"
        instance prim_type: float;
        instance prim_style: float;

        fn pixel() -> vec4 {
            // Match viewport to pixels
            let df = Df::viewport_px(pos * rect_size);

            let size = rect_size.x * dpi_factor;
            let x = 0.25 * size;
            let y = 0.25 * size;
            size = size / 2.;

            if prim_type == 0. {
                df.circle(x + 0.5 * size, y + 0.5 * size, 0.5 * size);
            } else if prim_type == 1. {
                df.arc(x, y + size, size, 0., PI / 2.);
            } else if prim_type == 2. {
                df.box(x, y, size, size, 0.15 * size);
            } else if prim_type == 3. {
                df.rect(x, y, size, size);
            } else if prim_type == 4. {
                df.hexagon(x + .5 * size, y + .5 * size, .5 * size);
            } else if prim_type == 5. {
                df.move_to(x, y);
                df.line_to(x + .25 * size, y + .3 * size);
                df.line_to(x + .5 * size, y + .15 * size);
                df.line_to(x + .75 * size, y + .8 * size);
                df.line_to(x + .25 * size, y + .9 * size);
                df.line_to(x + .1 * size, y + .5 * size);
            } else if prim_type == 6. {
                df.move_to(x, y);
                df.line_to(x + .25 * size, y + .3 * size);
                df.line_to(x + .5 * size, y + .15 * size);
                df.line_to(x + .75 * size, y + .8 * size);
                df.line_to(x + .25 * size, y + .9 * size);
                df.line_to(x + .1 * size, y + .5 * size);
                df.close_path();
            } else if prim_type == 7. {
                df.triangle(vec2(x, y + size), vec2(x + 0.5 * size, y), vec2(x + size, y + size));
            }

            if prim_style == 0. {
                df.fill(#f00);
            } else if prim_style == 1. {
                df.stroke(#0f0, 1.);
            } else if prim_style == 2. {
                df.stroke(#0f0, 5.);
            } else if prim_style == 3. {
                df.stroke(#0f0, 10.);
            } else if prim_style == 4. {
                df.fill_keep(#f00);
                df.stroke(#0f0, 1.);
            } else if prim_style == 5. {
                df.fill_keep(#f00);
                df.stroke(#0f0, 5.);
            } else if prim_style == 6. {
                df.fill_keep(#f00);
                df.stroke(#0f0, 10.);
            }

            return df.result;
        }"#
    ),
);

#[derive(Clone)]
#[repr(C)]
struct PrimitiveInsProps {
    primitive_type: f32,
    style: f32,
}

#[derive(Clone)]
#[repr(C)]
struct PrimitiveIns {
    base: QuadIns,
    prim_type: f32,
    prim_style: f32,
}

impl PrimitiveIns {
    fn draw(cx: &mut Cx, rect: Rect, props: PrimitiveInsProps) {
        cx.add_instances(
            &MAIN_SHADER,
            &[PrimitiveIns { base: QuadIns::from_rect(rect), prim_type: props.primitive_type, prim_style: props.style }],
        );
    }
}

pub struct PrimitivesExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    count: f32,
}

#[cfg(feature = "cef-server")]
fn get_resource_url_callback(url: &str, current_directory: &str) -> String {
    let path = format!("{}/wrflib/examples/shader_2d_primitives", &current_directory);
    let url = url.replace("http://localhost:3000", &path);
    url.to_string()
}

impl PrimitivesExampleApp {
    pub fn new(_: &mut Cx) -> Self {
        Self {
            window: Window {
                create_inner_size: Some(Vec2 { x: 1260., y: 660. }),
                #[cfg(feature = "cef")]
                create_cef_url: Some("http://localhost:3000/index.html".to_string()),
                #[cfg(feature = "cef-server")]
                get_resource_url_callback: Some(get_resource_url_callback),
                ..Window::default()
            },
            pass: Pass::default(),
            main_view: View::default(),
            count: 0.,
        }
    }

    pub fn handle(&mut self, _cx: &mut Cx, event: &mut Event) {
        match event {
            Event::Construct => {}
            Event::FingerMove(fm) => {
                self.count = fm.abs.x * 0.01;
            }
            _ => (),
        }
    }

    fn draw_grid(&mut self, cx: &mut Cx, bounds: Rect, cell_size: Vec2) {
        let color = vec4(0.5, 0.5, 0.5, 1.);
        let dark_color = vec4(0.25, 0.25, 0.25, 1.);
        let scale = 1.;

        let min_x = bounds.pos.x;
        let max_x = min_x + bounds.size.x;
        let min_y = bounds.pos.y;
        let max_y = min_y + bounds.size.y;

        let mut lines = vec![];

        let mut x = min_x;
        while x <= max_x {
            lines.push(DrawLines3dInstance::from_segment(vec3(x, min_y, 0.), vec3(x, max_y, 0.), color, scale));
            x += cell_size.x;
        }

        let mut x = min_x + 0.5 * cell_size.x;
        while x <= max_x {
            lines.push(DrawLines3dInstance::from_segment(vec3(x, min_y, 0.), vec3(x, max_y, 0.), dark_color, scale));
            x += cell_size.x;
        }

        let mut y = min_y;
        while y <= max_y {
            lines.push(DrawLines3dInstance::from_segment(vec3(min_x, y, 0.), vec3(max_x, y, 0.), color, scale));
            y += cell_size.y;
        }

        let mut y = min_y + 0.5 * cell_size.y;
        while y <= max_y {
            lines.push(DrawLines3dInstance::from_segment(vec3(min_x, y, 0.), vec3(max_x, y, 0.), dark_color, scale));
            y += cell_size.y;
        }

        DrawLines3d::draw(cx, &lines);
    }

    fn draw_primitives(&mut self, cx: &mut Cx, bounds: Rect, cell_size: Vec2) {
        let mut primitive_type = 0.;
        for y in (0..bounds.size.y as usize).step_by(cell_size.y as usize) {
            let mut style: f32 = 0.;
            for x in (0..bounds.size.x as usize).step_by(cell_size.x as usize) {
                let x = bounds.pos.x + x as f32;
                let y = bounds.pos.y + y as f32;
                PrimitiveIns::draw(cx, Rect { pos: vec2(x, y), size: cell_size }, PrimitiveInsProps { primitive_type, style });
                style += 1.;
            }
            primitive_type += 1.;
        }
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("0000"));
        self.main_view.begin_view(cx, Layout::default());

        TextIns::draw_str(
            cx,
            "Wrflib",
            cx.get_turtle_origin() + vec2(320., 20.),
            &TextInsProps { position_anchoring: TEXT_ANCHOR_CENTER_H, ..TextInsProps::DEFAULT },
        );

        TextIns::draw_str(
            cx,
            "Canvas",
            cx.get_turtle_origin() + vec2(940., 20.),
            &TextInsProps { position_anchoring: TEXT_ANCHOR_CENTER_H, ..TextInsProps::DEFAULT },
        );

        let bounds = Rect { pos: vec2(20., 40.), size: vec2(600., 600.) };

        let prim_count = 10;
        let cell_size = bounds.size / prim_count as f32;

        self.draw_grid(cx, bounds, cell_size);
        self.draw_primitives(cx, bounds, cell_size);

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
        cx.request_draw();
    }
}

main_app!(PrimitivesExampleApp);
