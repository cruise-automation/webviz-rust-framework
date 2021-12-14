// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use std::f32::consts::PI;

use wrflib::*;

/// Carefully chosen so that at the poles (all the way up or down) you can still rotate
/// nicely.
const EPSILON: f32 = 0.0001;

/// A nice article about how a 3D camera's look_at function works:
/// <https://www.scratchapixel.com/lessons/mathematics-physics-for-computer-graphics/lookat-function>
fn look_at(eye: Vec3, at: Vec3, up: Vec3) -> Mat4 {
    let forward = (eye - at).normalize();
    let left = Vec3::cross(up, forward).normalize();
    let up = Vec3::cross(forward, left);

    let mut matrix = Mat4::identity();
    matrix.v[0] = left.x;
    matrix.v[4] = left.y;
    matrix.v[8] = left.z;
    matrix.v[1] = up.x;
    matrix.v[5] = up.y;
    matrix.v[9] = up.z;
    matrix.v[2] = forward.x;
    matrix.v[6] = forward.y;
    matrix.v[10] = forward.z;
    matrix.v[12] = -left.dot(eye);
    matrix.v[13] = -up.dot(eye);
    matrix.v[14] = -forward.dot(eye);
    matrix
}

#[derive(Clone, Copy)]
pub struct SphericalAngles {
    phi: f32,
    theta: f32,
}

pub struct TrackingViewport3D {
    component_base: ComponentBase,
    pub pass: Pass,
    pub clear_color: Vec4,
    pub color_texture: Texture,
    pub depth_texture: Texture,
    pub view_2d: View,
    pub view_3d: View,
    pub measured_size: Vec2,
    pub camera_distance: f32,
    pub camera_phi_theta: SphericalAngles,
    pub camera_phi_theta_start: Option<SphericalAngles>,
    pub camera_target: Vec3,
    pub camera_target_offset: Vec3,
    pub camera_target_offset_start: Option<Vec3>,
}

impl Default for TrackingViewport3D {
    fn default() -> Self {
        Self {
            component_base: Default::default(),
            camera_distance: 50.,
            // Make_safe concept borrowed from ThreeJS:
            // https://github.com/mrdoob/three.js/blob/342946c8392639028da439b6dc0597e58209c696/src/math/Spherical.js#L43
            camera_phi_theta: SphericalAngles { phi: 0., theta: PI / 2. - EPSILON },
            measured_size: Default::default(),
            camera_phi_theta_start: Default::default(),
            camera_target: Default::default(),
            camera_target_offset: Default::default(),
            camera_target_offset_start: Default::default(),
            pass: Default::default(),
            clear_color: Default::default(),
            color_texture: Default::default(),
            depth_texture: Default::default(),
            view_3d: Default::default(),
            view_2d: Default::default(),
        }
    }
}

impl TrackingViewport3D {
    pub fn handle_viewport_2d(&mut self, cx: &mut Cx, event: &mut Event) -> Option<PassMatrixMode> {
        match event.hits(cx, &self.component_base, HitOpt::default()) {
            Event::FingerHover(_fe) => {
                // cx.set_hover_mouse_cursor(MouseCursor::Move);
            }
            // traditional mouse down
            Event::FingerDown(fe) => {
                // cx.set_down_mouse_cursor(MouseCursor::Move);
                if fe.button == MouseButton::Left {
                    self.camera_target_offset_start = Some(self.camera_target_offset);
                } else if fe.button == MouseButton::Right {
                    self.camera_phi_theta_start = Some(self.camera_phi_theta);
                }
            }
            // traditional mouse up
            Event::FingerUp(_fe) => {
                self.camera_phi_theta_start = None;
                self.camera_target_offset_start = None;
            }
            Event::FingerScroll(fe) => {
                let min_distance = 1.0; // a little more than near
                let max_distance = 900.; // a little less than far
                let zoom_speed = (self.camera_distance * (PI / 4.) / max_distance).sin().abs() / 2.0;
                self.camera_distance = (self.camera_distance + fe.scroll.y * zoom_speed).max(min_distance).min(max_distance);
                return Some(self.pass_set_matrix_mode(cx));
            }
            Event::FingerMove(fe) => {
                // Using standard makeSafe approach to clamp to slightly less than the limits for phi/theta
                // Concept borrowed from ThreeJS:
                // https://github.com/mrdoob/three.js/blob/342946c8392639028da439b6dc0597e58209c696/src/math/Spherical.js#L43
                if let Some(camera_phi_theta_start) = self.camera_phi_theta_start {
                    let rotate_speed = 1. / 175.;
                    self.camera_phi_theta = SphericalAngles {
                        phi: (camera_phi_theta_start.phi + (fe.abs.x - fe.abs_start.x) * rotate_speed) % (PI * 2.),
                        theta: (camera_phi_theta_start.theta + (fe.abs.y - fe.abs_start.y) * rotate_speed)
                            .clamp(-PI / 2. + EPSILON, PI / 2. - EPSILON),
                    };
                    return Some(self.pass_set_matrix_mode(cx));
                } else if let Some(camera_target_offset_start) = self.camera_target_offset_start {
                    // TODO(Shobhit): Whenever we do Orthographic view properly, we need to adjust the panning accordingly
                    // We would need to consider viewable area's width and height into consideration just like how
                    // worldview does it:
                    // https://git.io/J0wsP
                    // Please refer some more discussion about this here:
                    // https://github.robot.car/cruise/exviz/pull/107#discussion_r932946
                    let pan_speed = 0.8;
                    // Normalize using the height of the viewport and the camera distance, since those determine the field of view
                    // intersecting with the camera target.
                    let mouse_offset = (fe.rel - fe.rel_start) / self.measured_size.y * self.camera_distance * pan_speed;

                    // We need to calculate the value of camera target offset.
                    // For that we create a rotation matrix that disregards x and y axis rotations,
                    // and only uses PHI of the spherical coordinate value camera_phi_theta (the rotation),
                    // thereafter we translate it on x/y based on relative offset calculated mouse movements.
                    // Finally, so that we don't forget about the previous camera target offsets,
                    // we add camera_target_offset_start so that we don't start from beginning in every interaction.
                    self.camera_target_offset = Mat4::rotation(0., 0., (-self.camera_phi_theta.phi).to_degrees())
                        .transform_vec4(vec4(mouse_offset.y, mouse_offset.x, 0., 1.0))
                        .to_vec3()
                        + camera_target_offset_start;

                    return Some(self.pass_set_matrix_mode(cx));
                }
            }
            _ => (),
        }

        None
    }

    pub fn get_matrix_projection(&self) -> PassMatrixMode {
        let eye = self.camera_distance
            * vec3(
                -self.camera_phi_theta.phi.cos() * self.camera_phi_theta.theta.cos(),
                self.camera_phi_theta.phi.sin() * self.camera_phi_theta.theta.cos(),
                self.camera_phi_theta.theta.sin(),
            );

        PassMatrixMode::Projection {
            fov_y: 40.0,
            near: 0.1,
            far: 1000.0,
            cam: look_at(
                eye + self.camera_target + self.camera_target_offset,
                self.camera_target + self.camera_target_offset,
                vec3(0., 0., 1.),
            ),
        }
    }

    fn pass_set_matrix_mode(&mut self, cx: &mut Cx) -> PassMatrixMode {
        let matrix_mode = self.get_matrix_projection();
        self.pass.set_matrix_mode(cx, matrix_mode.clone());
        matrix_mode
    }

    /// TODO(JP): This is kind of exploiting a potential bug in the framework.. We don't clean up [`Pass`]es,
    /// so if we just don't call [`Pass::begin_pass`] then it will happily keep on rendering. Is this a bug
    /// or a feature? I'm not sure.. See [`Pass::begin_pass`] for more thoughts.
    #[must_use]
    pub fn skip_viewport_3d(&mut self, cx: &mut Cx) -> bool {
        // We have to manually check if the size has changed. See [`Pass:begin_pass`] for more info.
        if self.measured_size != vec2(cx.get_width_total(), cx.get_height_total()) {
            return false;
        }
        self.draw_viewport_2d(cx);
        true
    }

    pub fn begin_viewport_3d(&mut self, cx: &mut Cx) {
        self.draw_viewport_2d(cx);

        self.pass.begin_pass_without_textures(cx);

        self.pass.set_size(cx, self.measured_size);
        let color_texture_handle = self.color_texture.get_color(cx);
        self.pass.add_color_texture(cx, color_texture_handle, ClearColor::ClearWith(self.clear_color));
        let depth_texture_handle = self.depth_texture.get_depth(cx);
        self.pass.set_depth_texture(cx, depth_texture_handle, ClearDepth::ClearWith(1.0));

        self.view_3d.begin_view(cx, Layout::default());
    }

    pub fn end_viewport_3d(&mut self, cx: &mut Cx) -> PassMatrixMode {
        let matrix_mode = self.pass_set_matrix_mode(cx);

        self.view_3d.end_view(cx);
        self.pass.end_pass(cx);

        matrix_mode
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
