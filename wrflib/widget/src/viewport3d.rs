// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// a bunch o buttons to select the world
use wrflib::*;

pub struct Viewport3D {
    component_base: ComponentBase,
    pub pass: Pass,
    pub clear_color: Vec4,
    pub color_texture: Texture,
    pub depth_texture: Texture,
    pub view_2d: View,
    pub view_3d: View,
    pub measured_size: Vec2,
    pub camera_center: Vec3,
    pub camera_pos: Vec3,
    pub camera_rot: Vec3,
    pub camera_start: Option<(Vec3, Vec3)>,
}

impl Default for Viewport3D {
    fn default() -> Self {
        Self {
            component_base: Default::default(),
            camera_center: Vec3 { x: 0.0, y: 0.0, z: 1.1 + 1.5 },
            camera_pos: Vec3 { x: 0.0, y: -0.5, z: -1.1 },
            camera_rot: Default::default(),
            clear_color: Default::default(),
            pass: Default::default(),
            camera_start: Default::default(),
            color_texture: Default::default(),
            depth_texture: Default::default(),
            view_3d: Default::default(),
            view_2d: Default::default(),
            measured_size: Default::default(),
        }
    }
}

impl Viewport3D {
    pub fn handle_viewport_2d(&mut self, cx: &mut Cx, event: &mut Event) {
        match event.hits(cx, &self.component_base, HitOpt::default()) {
            Event::FingerHover(_fe) => {
                cx.set_hover_mouse_cursor(MouseCursor::Move);
            }
            Event::FingerDown(_fe) => {
                cx.set_down_mouse_cursor(MouseCursor::Move);
                self.camera_start = Some((self.camera_pos, self.camera_rot));
            }
            Event::FingerUp(_fe) => {}
            Event::FingerScroll(fe) => {
                self.camera_pos.z += fe.scroll.y / 300.0;
                self.camera_center.z = -self.camera_pos.z + 1.5;
                self.pass_set_matrix_mode(cx);
            }
            Event::FingerMove(fe) => {
                if let Some((_pos, rot)) = self.camera_start {
                    self.camera_rot =
                        Vec3 { x: rot.x + (fe.abs.y - fe.abs_start.y), y: rot.y + (fe.abs.x - fe.abs_start.x), z: rot.z };
                    self.pass_set_matrix_mode(cx)
                }
            }
            _ => (),
        }
    }

    pub fn pass_set_matrix_mode(&mut self, cx: &mut Cx) {
        //self.pass.set_matrix_mode(cx, PassMatrixMode::Ortho);

        self.pass.set_matrix_mode(
            cx,
            PassMatrixMode::Projection {
                fov_y: 40.0,
                near: 0.1,
                far: 1000.0,
                cam: Mat4::txyz_s_ry_rx_txyz(
                    self.camera_pos + self.camera_center,
                    1.0,
                    self.camera_rot.y,
                    self.camera_rot.x,
                    -self.camera_center,
                ),
            },
        );
    }

    pub fn begin_viewport_3d(&mut self, cx: &mut Cx) {
        self.draw_viewport_2d(cx);

        self.pass.begin_pass_without_textures(cx);
        self.pass.set_size(cx, self.measured_size);
        self.pass_set_matrix_mode(cx);
        let color_texture_handle = self.color_texture.get_color(cx);
        self.pass.add_color_texture(cx, color_texture_handle, ClearColor::ClearWith(self.clear_color));
        let depth_texture_handle = self.depth_texture.get_depth(cx);
        self.pass.set_depth_texture(cx, depth_texture_handle, ClearDepth::ClearWith(1.0));

        let _ = self.view_3d.begin_view(cx, Layout::default());
    }

    pub fn end_viewport_3d(&mut self, cx: &mut Cx) {
        self.view_3d.end_view(cx);
        self.pass.end_pass(cx);
    }

    fn draw_viewport_2d(&mut self, cx: &mut Cx) {
        self.view_2d.begin_view(cx, Layout::default());
        // blit the texture to a view rect
        self.measured_size = vec2(cx.get_width_total(), cx.get_height_total());
        let color_texture_handle = self.color_texture.get_color(cx);
        let area = ImageIns::draw(cx, Rect { pos: cx.get_turtle_origin(), size: self.measured_size }, color_texture_handle);
        self.component_base.register_component_area(cx, area);

        self.view_2d.end_view(cx);
    }
}
