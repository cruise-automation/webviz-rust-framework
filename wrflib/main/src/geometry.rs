// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! Instance geometries, for rendering the same shape multiple times.

use std::f32::consts::PI;

use crate::cx::*;

pub(crate) fn generate_default_geometries(cx: &mut Cx) {
    // geometry_id: 0
    declare_geometry(cx, GeometryGen::from_quad_2d(0.0, 0.0, 1.0, 1.0));
    // geometry_id: 1
    declare_geometry(cx, GeometryGen::from_cube_3d(1.0, 1.0, 1.0, 1, 1, 1));
    // geometry_id: 2
    declare_geometry(cx, GeometryGen::from_sphere_3d(30, 30, 0.5));
    // geometry_id: 3
    declare_geometry(cx, GeometryGen::from_cylinder_3d(30, false));
    // geometry_id: 4
    declare_geometry(cx, GeometryGen::from_cylinder_3d(30, true));
}
/// The 2d quad [`Geometry`], for instancing rectangles, e.g. using [`crate::QuadIns`].
pub const GEOM_QUAD2D: Geometry = Geometry { geometry_id: 0 };
/// The 3d cube [`Geometry`], for instancing rectangles, e.g. using [`crate::CubeIns`].
pub const GEOM_CUBE3D: Geometry = Geometry { geometry_id: 1 };
/// The 3d sphere [`Geometry`], for instancing spheres.
pub const GEOM_SPHERE3D: Geometry = Geometry { geometry_id: 2 };
/// The 3d cylinder [`Geometry`], for instancing cylinders.
pub const GEOM_CYLINDER3D: Geometry = Geometry { geometry_id: 3 };
/// The 3d cone [`Geometry`], for instancing cones.
pub const GEOM_CONE3D: Geometry = Geometry { geometry_id: 4 };

/// Some shader fields to use when using [`GEOM_CUBE3D`].
///
/// TODO(JP): Should we make this part of the [`Geometry`] instead so they are
/// always together, and can be automatically included?
pub const GEOM_3D_SHADER_FIELDS: CodeFragment = code_fragment!(
    r#"
    geometry geom_pos: vec3;
    geometry geom_id: float;
    geometry geom_normal: vec3;
    geometry geom_uv: vec2;
"#
);

/// A pointer to a [`CxGeometry`] (indexed in [`Cx::geometries`] using [`Geometry::geometry_id`]),
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Geometry {
    pub(crate) geometry_id: usize,
}

/// The base fields used for instance rendering. Created using [`GeometryGen`].
pub(crate) struct CxGeometry {
    pub(crate) indices: Vec<u32>,
    pub(crate) vertices: Vec<f32>,
    pub(crate) dirty: bool,
    pub(crate) platform: CxPlatformGeometry,
}

fn declare_geometry(cx: &mut Cx, gen: GeometryGen) {
    cx.geometries.push(CxGeometry {
        indices: gen.indices,
        vertices: gen.vertices,
        dirty: true,
        platform: CxPlatformGeometry::default(),
    });
}

/// Generated geometry data, used for instanced renderen. For example, you can define that a
/// quad has 4 vertices (for use in e.g. [`crate::QuadIns`]), so you don't have to manually create them
/// every time you want to render a quad.
#[derive(Clone, Debug, Default, PartialEq)]
struct GeometryGen {
    vertices: Vec<f32>, // e.g. vec4 pos, vec3 normal, vec2 uv
    indices: Vec<u32>,
}

#[derive(Clone, Copy)]
enum GeometryAxis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl GeometryGen {
    fn from_quad_2d(x1: f32, y1: f32, x2: f32, y2: f32) -> GeometryGen {
        let mut g = Self::default();
        g.add_quad_2d(x1, y1, x2, y2);
        g
    }

    fn from_cube_3d(
        width: f32,
        height: f32,
        depth: f32,
        width_segments: usize,
        height_segments: usize,
        depth_segments: usize,
    ) -> GeometryGen {
        let mut g = Self::default();
        g.add_cube_3d(width, height, depth, width_segments, height_segments, depth_segments);
        g
    }

    fn from_sphere_3d(num_parallels: u32, num_meridians: u32, radius: f32) -> GeometryGen {
        let mut g = Self::default();
        g.add_sphere_3d(num_parallels, num_meridians, radius);
        g
    }

    fn from_cylinder_3d(num_segments: u32, is_cone: bool) -> GeometryGen {
        let mut g = Self::default();
        g.add_cylinder_3d(num_segments, is_cone);
        g
    }

    // requires pos:vec2 normalized layout
    fn add_quad_2d(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let vertex_offset = self.vertices.len() as u32;
        self.vertices.push(x1);
        self.vertices.push(y1);
        self.vertices.push(x2);
        self.vertices.push(y1);
        self.vertices.push(x2);
        self.vertices.push(y2);
        self.vertices.push(x1);
        self.vertices.push(y2);
        self.indices.push(vertex_offset);
        self.indices.push(vertex_offset + 1);
        self.indices.push(vertex_offset + 2);
        self.indices.push(vertex_offset + 2);
        self.indices.push(vertex_offset + 3);
        self.indices.push(vertex_offset);
    }

    // requires pos:vec3, id:float, normal:vec3, uv:vec2 layout
    fn add_cube_3d(
        &mut self,
        width: f32,
        height: f32,
        depth: f32,
        width_segments: usize,
        height_segments: usize,
        depth_segments: usize,
    ) {
        self.add_plane_3d(
            GeometryAxis::Z,
            GeometryAxis::Y,
            GeometryAxis::X,
            -1.0,
            -1.0,
            depth,
            height,
            width,
            depth_segments,
            height_segments,
            0.0,
        );
        self.add_plane_3d(
            GeometryAxis::Z,
            GeometryAxis::Y,
            GeometryAxis::X,
            1.0,
            -1.0,
            depth,
            height,
            -width,
            depth_segments,
            height_segments,
            1.0,
        );
        self.add_plane_3d(
            GeometryAxis::X,
            GeometryAxis::Z,
            GeometryAxis::Y,
            1.0,
            1.0,
            width,
            depth,
            height,
            width_segments,
            depth_segments,
            2.0,
        );
        self.add_plane_3d(
            GeometryAxis::X,
            GeometryAxis::Z,
            GeometryAxis::Y,
            1.0,
            -1.0,
            width,
            depth,
            -height,
            width_segments,
            depth_segments,
            3.0,
        );
        self.add_plane_3d(
            GeometryAxis::X,
            GeometryAxis::Y,
            GeometryAxis::Z,
            1.0,
            -1.0,
            width,
            height,
            depth,
            width_segments,
            height_segments,
            4.0,
        );
        self.add_plane_3d(
            GeometryAxis::X,
            GeometryAxis::Y,
            GeometryAxis::Z,
            -1.0,
            -1.0,
            width,
            height,
            -depth,
            width_segments,
            height_segments,
            5.0,
        );
    }

    fn add_vertex(&mut self, position: Vec3, normal: Vec3, id: f32, uv: Vec2) {
        self.vertices.push(position.x);
        self.vertices.push(position.y);
        self.vertices.push(position.z);
        self.vertices.push(normal.x);
        self.vertices.push(normal.y);
        self.vertices.push(normal.z);
        self.vertices.push(id);
        self.vertices.push(uv.x);
        self.vertices.push(uv.y);
    }

    // requires pos:vec3, id:float, normal:vec3, uv:vec2 layout
    fn add_sphere_3d(&mut self, num_parallels: u32, num_meridians: u32, radius: f32) {
        // north pole
        self.add_vertex(vec3(0., 0., radius), vec3(0., 0., radius), 0., vec2(0., 0.));

        // south pole
        self.add_vertex(vec3(0., 0., -radius), vec3(0., 0., -radius), 0., vec2(0., 0.));

        let mut point_count: u32 = 2;

        for i in 0..num_parallels {
            for j in 0..num_meridians {
                let phi = (((i + 1) as f32) / ((num_parallels + 1) as f32)) * PI;
                let z = radius * phi.cos();
                let width = radius * phi.sin();
                let theta = (j as f32 * 2. * PI) / (num_meridians as f32);
                let x = width * theta.cos();
                let y = width * theta.sin();

                self.add_vertex(vec3(x, y, z), vec3(x, y, z), 0., vec2(0., 0.));
                point_count += 1;

                if j > 0 {
                    let prev_meridian: u32 = if i == 0 { 0 } else { point_count - 1 - num_meridians };
                    self.indices.push(point_count - 2);
                    self.indices.push(point_count - 1);
                    self.indices.push(prev_meridian);

                    if i > 0 {
                        self.indices.push(point_count - 2);
                        self.indices.push(prev_meridian - 1);
                        self.indices.push(prev_meridian);
                    }
                }
            }

            let prev_meridian: u32 = if i == 0 { 0 } else { point_count - 2 * num_meridians };
            self.indices.push(point_count - 1);
            self.indices.push(point_count - num_meridians);
            self.indices.push(prev_meridian);

            if i > 0 {
                self.indices.push(point_count - 1);
                self.indices.push(point_count - num_meridians - 1);
                self.indices.push(prev_meridian);
            }
        }

        // connect last parallel to south pole
        for j in 0..num_meridians {
            let pt = point_count - num_meridians + j;
            let prev_pt = if j == 0 { point_count - 1 } else { pt - 1 };
            self.indices.push(pt);
            self.indices.push(prev_pt);
            self.indices.push(1);
        }
    }

    fn add_cylinder_3d(&mut self, num_segments: u32, is_cone: bool) {
        // "poles" are the centers of top/bottom faces
        // north pole
        self.add_vertex(vec3(0., 0., 0.5), vec3(0., 1., 0.), 0., vec2(0., 0.));
        // south pole
        self.add_vertex(vec3(0., 0., -0.5), vec3(0., -1., 0.), 0., vec2(0., 0.));

        // Keep side faces separate from top/bottom to improve appearance for semi-transparent colors.
        // We don't have a good approach to transparency right now but this is a small improvement over mixing the faces.
        let mut side_faces = Vec::<u32>::new();
        let mut end_cap_faces = Vec::<u32>::new();

        let mut point_count = 0;

        for i in 0..num_segments {
            let theta = (2. * PI * i as f32) / num_segments as f32;
            let x = 0.5 * theta.cos();
            let y = 0.5 * theta.sin();

            self.add_vertex(vec3(x, y, 0.5), vec3(x, y, 0.5), 0., vec2(0., 0.));
            self.add_vertex(vec3(x, y, -0.5), vec3(x, y, -0.5), 0., vec2(0., 0.));

            point_count += 2;

            let bottom_left_pt = point_count - 1;
            let top_right_pt = if is_cone {
                0
            } else if i + 1 == num_segments {
                2
            } else {
                point_count
            };
            let bottom_right_pt = if i + 1 == num_segments { 3 } else { point_count + 1 };

            side_faces.extend_from_slice(&[bottom_left_pt, top_right_pt, bottom_right_pt]);
            end_cap_faces.extend_from_slice(&[bottom_left_pt, bottom_right_pt, 1]);

            if !is_cone {
                let top_left_pt = point_count - 2;
                side_faces.extend_from_slice(&[top_left_pt, bottom_left_pt, top_right_pt]);
                end_cap_faces.extend_from_slice(&[top_left_pt, top_right_pt, 0]);
            }
        }

        self.indices.extend_from_slice(&side_faces[..]);
        self.indices.extend_from_slice(&end_cap_faces[..]);
    }

    // requires pos:vec3, id:float, normal:vec3, uv:vec2 layout
    // Clippy TODO
    #[warn(clippy::many_single_char_names)]
    fn add_plane_3d(
        &mut self,
        u: GeometryAxis,
        v: GeometryAxis,
        w: GeometryAxis,
        udir: f32,
        vdir: f32,
        width: f32,
        height: f32,
        depth: f32,
        grid_x: usize,
        grid_y: usize,
        id: f32,
    ) {
        let segment_width = width / (grid_x as f32);
        let segment_height = height / (grid_y as f32);
        let width_half = width / 2.0;
        let height_half = height / 2.0;
        let depth_half = depth / 2.0;
        let grid_x1 = grid_x + 1;
        let grid_y1 = grid_y + 1;

        let vertex_offset = self.vertices.len() / 9;

        for iy in 0..grid_y1 {
            let y = (iy as f32) * segment_height - height_half;

            for ix in 0..grid_x1 {
                let x = (ix as f32) * segment_width - width_half;
                let off = self.vertices.len();
                self.vertices.push(0.0);
                self.vertices.push(0.0);
                self.vertices.push(0.0);

                self.vertices[off + u as usize] = x * udir;
                self.vertices[off + v as usize] = y * vdir;
                self.vertices[off + w as usize] = depth_half;

                self.vertices.push(id);
                let off = self.vertices.len();
                self.vertices.push(0.0);
                self.vertices.push(0.0);
                self.vertices.push(0.0);
                self.vertices[off + w as usize] = if depth > 0.0 { 1.0 } else { -1.0 };

                self.vertices.push((ix as f32) / (grid_x as f32));
                self.vertices.push(1.0 - (iy as f32) / (grid_y as f32));
            }
        }

        for iy in 0..grid_y {
            for ix in 0..grid_x {
                let a = vertex_offset + ix + grid_x1 * iy;
                let b = vertex_offset + ix + grid_x1 * (iy + 1);
                let c = vertex_offset + (ix + 1) + grid_x1 * (iy + 1);
                let d = vertex_offset + (ix + 1) + grid_x1 * iy;
                self.indices.push(a as u32);
                self.indices.push(b as u32);
                self.indices.push(d as u32);
                self.indices.push(b as u32);
                self.indices.push(c as u32);
                self.indices.push(d as u32);
            }
        }
    }
}
