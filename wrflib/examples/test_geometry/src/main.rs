// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;
use wrflib_widget::*;

#[repr(C)]
#[derive(Debug, Clone)]
struct Instance {
    normal: Vec3,
}

static SHADER: Shader = Shader {
    code_to_concatenate: &[
        Cx::STD_SHADER,
        Geometry3d::SHADER,
        code_fragment!(
            r#"
            // TODO(JP): Make it so you can just render a single geometry without having to use instancing.
            // Since we still need instancing we need a dummy value here.
            instance _dummy: float;

            fn vertex() -> vec4 {
                return camera_projection * camera_view * vec4(geom_pos, 1.);
            }

            fn pixel() -> vec4 {
                let lightPosition = vec3(20.,0.,30.);
                let lightDirection = normalize(geom_pos - lightPosition);
                return vec4(vec3(clamp(dot(-lightDirection, geom_normal), 0.2, 1.0)),1.0);
            }
            "#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub struct GeometryExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    viewport_3d: Viewport3D,
    num_sides: u32,
    timer: Timer,
}

const VIEWPORT_PROPS: Viewport3DProps =
    Viewport3DProps { initial_camera_position: Coordinates::Cartesian(vec3(5., 0., 5.)), ..Viewport3DProps::DEFAULT };

impl GeometryExampleApp {
    pub fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        self.viewport_3d.handle(cx, event);

        match event {
            Event::Construct => self.timer = cx.start_timer(0.2, true),
            Event::Timer(te) => {
                if self.timer.is_timer(te) {
                    self.num_sides += 1;
                    if self.num_sides > 10 {
                        self.num_sides = 0;
                    }
                    cx.request_draw();
                }
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("300"));
        self.main_view.begin_view(cx, Layout::default());

        // TODO(JP): This creates a GPU buffer since the old one isn't released yet at this point, which
        // causes us to oscillate between two GPU buffers. Not the end of the world but not great.
        let gpu_geometry = GpuGeometry::new(cx, Geometry3d::sphere(self.num_sides + 3, self.num_sides + 3, 0.5));
        self.viewport_3d.begin_draw(cx, VIEWPORT_PROPS);
        cx.add_mesh_instances(&SHADER, &[0], gpu_geometry);
        self.viewport_3d.end_draw(cx);

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(GeometryExampleApp);
