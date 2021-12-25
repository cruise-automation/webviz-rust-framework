// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! Managing [GPU shaders](https://en.wikipedia.org/wiki/Shader).

use crate::cx::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use wrflib_shader_compiler::ty::Ty;
use wrflib_shader_compiler::{Decl, ShaderAst};

/// Contains all information necessary to build a shader. Create using [`Cx::define_shader`].
pub struct Shader {
    /// The default [`Geometry`] that we will draw with, if any. Can be overridden using [`DrawCallProps::geometry`].
    default_geometry: Option<Geometry>,
    /// A bunch of [`CodeFragment`]s that will get concatenated.
    base_code_fragments: &'static [CodeFragment],
    /// The main [`CodeFragment`] that will be concatanated after the `base_code_fragments`.
    main_code_fragment: CodeFragment,
    /// The id of the shader (index into [`Cx::shaders`]), or [`UNCOMPILED_SHADER_ID`] if uninitialized.
    shader_id: AtomicUsize,
}
const UNCOMPILED_SHADER_ID: usize = usize::MAX;

/// Contains information of a [`CxShader`] of what instances, instances, textures
/// and so on it contains. That information can then be used to modify a [`Shader`
/// or [`DrawCall`].
#[derive(Debug, Default, Clone)]
pub struct CxShaderMapping {
    /// Contains information about the special "rect_pos" and "rect_size" fields.
    /// See [`RectInstanceProps`].
    pub rect_instance_props: RectInstanceProps,
    /// Special structure for user-level uniforms.
    pub user_uniform_props: UniformProps,
    /// Special structure for reading/editing instance properties.
    pub instance_props: InstanceProps,
    /// Special structure for reading/editing geometry properties.
    pub geometry_props: InstanceProps,
    /// Raw definition of all textures.
    pub textures: Vec<PropDef>,
    /// Raw definition of all geometries.
    pub geometries: Vec<PropDef>,
    /// Raw definition of all instances.
    pub instances: Vec<PropDef>,
    /// Raw definition of all user-level uniforms.
    pub user_uniforms: Vec<PropDef>,
    /// Raw definition of all framework-level uniforms that get set per [`DrawCall`].
    pub draw_uniforms: Vec<PropDef>,
    /// Raw definition of all framework-level uniforms that get set per [`View`].
    pub view_uniforms: Vec<PropDef>,
    /// Raw definition of all framework-level uniforms that get set per [`Pass`].
    pub pass_uniforms: Vec<PropDef>,
}

impl CxShaderMapping {
    pub fn from_shader_ast(shader_ast: ShaderAst) -> Self {
        let mut instances = Vec::new();
        let mut geometries = Vec::new();
        let mut user_uniforms = Vec::new();
        let mut draw_uniforms = Vec::new();
        let mut view_uniforms = Vec::new();
        let mut pass_uniforms = Vec::new();
        let mut textures = Vec::new();
        for decl in shader_ast.decls {
            match decl {
                Decl::Geometry(decl) => {
                    let prop_def = PropDef { name: decl.ident.to_string(), ty: decl.ty_expr.ty.borrow().clone().unwrap() };
                    geometries.push(prop_def);
                }
                Decl::Instance(decl) => {
                    let prop_def = PropDef { name: decl.ident.to_string(), ty: decl.ty_expr.ty.borrow().clone().unwrap() };
                    instances.push(prop_def);
                }
                Decl::Uniform(decl) => {
                    let prop_def = PropDef { name: decl.ident.to_string(), ty: decl.ty_expr.ty.borrow().clone().unwrap() };
                    match decl.block_ident {
                        Some(bi) if bi.with(|string| string == "draw") => {
                            draw_uniforms.push(prop_def);
                        }
                        Some(bi) if bi.with(|string| string == "view") => {
                            view_uniforms.push(prop_def);
                        }
                        Some(bi) if bi.with(|string| string == "pass") => {
                            pass_uniforms.push(prop_def);
                        }
                        None => {
                            user_uniforms.push(prop_def);
                        }
                        _ => (),
                    }
                }
                Decl::Texture(decl) => {
                    let prop_def = PropDef { name: decl.ident.to_string(), ty: decl.ty_expr.ty.borrow().clone().unwrap() };
                    textures.push(prop_def);
                }
                _ => (),
            }
        }

        CxShaderMapping {
            rect_instance_props: RectInstanceProps::construct(&instances),
            user_uniform_props: UniformProps::construct(&user_uniforms),
            instance_props: InstanceProps::construct(&instances),
            geometry_props: InstanceProps::construct(&geometries),
            textures,
            instances,
            geometries,
            pass_uniforms,
            view_uniforms,
            draw_uniforms,
            user_uniforms,
        }
    }
}

/// The raw definition of an input property to a [`Shader`].
#[derive(Debug, Clone, Hash, PartialEq)]
pub struct PropDef {
    pub name: String,
    pub ty: Ty,
}

/// Contains information about the special "rect_pos" and "rect_size" fields.
/// These fields describe the typical rectangles drawn in [`crate::QuadIns`]. It is
/// useful to have generalized access to them, so we can e.g. move a whole bunch
/// of rectangles at the same time, e.g. for alignment in [`CxTurtle`].
/// TODO(JP): We might want to consider instead doing bulk moves using [`DrawCall`-
/// or [`View`]-level uniforms.
#[derive(Debug, Default, Clone)]
pub struct RectInstanceProps {
    pub rect_pos: Option<usize>,
    pub rect_size: Option<usize>,
}
impl RectInstanceProps {
    pub fn construct(instances: &[PropDef]) -> RectInstanceProps {
        let mut rect_pos = None;
        let mut rect_size = None;
        let mut slot = 0;
        for inst in instances {
            match inst.name.as_ref() {
                "rect_pos" => rect_pos = Some(slot),
                "rect_size" => rect_size = Some(slot),
                _ => (),
            }
            slot += inst.ty.size(); //sg.get_type_slots(&inst.ty);
        }
        RectInstanceProps { rect_pos, rect_size }
    }
}

/// Represents an "instance" GPU input in a [`Shader`].
///
/// TODO(JP): Merge this into [`NamedProp`].
#[derive(Debug, Clone)]
pub struct InstanceProp {
    pub name: String,
    pub ty: Ty,
    pub offset: usize,
    pub slots: usize,
}

/// Represents all "instance" GPU inputs in a [`Shader`].
///
/// TODO(JP): Merge this into [`NamedProps`].
#[derive(Debug, Default, Clone)]
pub struct InstanceProps {
    pub props: Vec<InstanceProp>,
    pub total_slots: usize,
}

/// Represents a "uniform" GPU input in a [`Shader`].
///
/// TODO(JP): Merge this into [`NamedProp`].
#[derive(Debug, Clone)]
pub struct UniformProp {
    pub name: String,
    pub ty: Ty,
    pub offset: usize,
    pub slots: usize,
}

/// Represents all "uniform" GPU inputs in a [`Shader`].
///
/// TODO(JP): Merge this into [`NamedProps`].
#[derive(Debug, Default, Clone)]
pub struct UniformProps {
    pub props: Vec<UniformProp>,
    pub total_slots: usize,
}

/// A generic representation of any kind of [`Shader`] input (instance/uniform/geometry).
#[derive(Debug, Clone)]
pub struct NamedProp {
    pub name: String,
    pub ty: Ty,
    pub offset: usize,
    pub slots: usize,
}

/// A generic representation of a list of [`Shader`] inputs (instance/uniform/geometry).
#[derive(Debug, Default, Clone)]
pub struct NamedProps {
    pub props: Vec<NamedProp>,
    pub total_slots: usize,
}

impl NamedProps {
    pub fn construct(in_props: &[PropDef]) -> NamedProps {
        let mut offset = 0;
        let out_props = in_props
            .iter()
            .map(|prop| {
                let slots = prop.ty.size();
                let prop = NamedProp { ty: prop.ty.clone(), name: prop.name.clone(), offset, slots };
                offset += slots;
                prop
            })
            .collect();
        NamedProps { props: out_props, total_slots: offset }
    }
}

impl InstanceProps {
    pub fn construct(in_props: &[PropDef]) -> InstanceProps {
        let mut offset = 0;
        let out_props = in_props
            .iter()
            .map(|prop| {
                let slots = prop.ty.size();
                let prop = InstanceProp { ty: prop.ty.clone(), name: prop.name.clone(), offset, slots };
                offset += slots;
                prop
            })
            .collect();
        InstanceProps { props: out_props, total_slots: offset }
    }
}

impl UniformProps {
    pub fn construct(in_props: &[PropDef]) -> UniformProps {
        let mut offset = 0;
        let out_props = in_props
            .iter()
            .map(|prop| {
                let slots = prop.ty.size();
                let prop = UniformProp { ty: prop.ty.clone(), name: prop.name.clone(), offset, slots };
                offset += slots;
                prop
            })
            .collect();
        UniformProps { props: out_props, total_slots: offset }
    }

    pub fn find_zbias_uniform_prop(&self) -> Option<usize> {
        for prop in &self.props {
            if prop.name == "zbias" {
                return Some(prop.offset);
            }
        }
        None
    }
}

/// The actual shader information, which gets stored on [`Cx`]. Once compiled the
/// [`ShaderAst`] will be removed, and the [`CxPlatformShader`] (platform-specific
/// part of the compiled shader) gets set.
#[derive(Default, Clone)]
pub struct CxShader {
    pub name: String,
    pub default_geometry: Option<Geometry>,
    pub(crate) platform: Option<CxPlatformShader>,
    pub mapping: CxShaderMapping,
    pub shader_ast: Option<ShaderAst>,
}

impl Cx {
    /// Define a new shader.
    ///
    /// Pass in a [`Geometry`] which gets used for instancing (e.g. a quad or a
    /// cube).
    ///
    /// The different [`CodeFragment`]s are appended together (but preserving their
    /// filename/line/column information for error messages). They are split out
    /// into `base_code_fragments` and `main_code_fragment` purely for
    /// convenience. (We could instead have used a single [`slice`] but they are
    /// annoying to get through concatenation..)
    ///
    /// TODO(JP): Would be good to instead compile shaders beforehand, ie. during
    /// compile time. Should look into that at some point.
    pub const fn define_shader(
        default_geometry: Option<Geometry>,
        base_code_fragments: &'static [CodeFragment],
        main_code_fragment: CodeFragment,
    ) -> Shader {
        Shader { default_geometry, base_code_fragments, main_code_fragment, shader_id: AtomicUsize::new(UNCOMPILED_SHADER_ID) }
    }

    /// Get an individual [`Shader`] from a static [`Shader`].
    ///
    /// For more information on what [`LocationHash`] is used for here, see [`Shader`].
    pub(crate) fn get_shader_id(&mut self, shader: &'static Shader) -> usize {
        let shader_id = shader.shader_id.load(Ordering::Relaxed);
        if shader_id != UNCOMPILED_SHADER_ID {
            shader_id
        } else {
            // Use the main_code_fragment's location as the shader name.
            let shader_name = format!(
                "{}:{}:{}",
                shader.main_code_fragment.filename, shader.main_code_fragment.line, shader.main_code_fragment.col
            );
            let code_fragments: Vec<&CodeFragment> =
                shader.base_code_fragments.iter().chain([&shader.main_code_fragment]).collect();
            let shader_ast = self.shader_ast_generator.generate_shader_ast(&shader_name, code_fragments);

            let shader_id = self.shaders.len();
            self.shaders.push(CxShader {
                name: shader_name,
                default_geometry: shader.default_geometry,
                mapping: CxShaderMapping::from_shader_ast(shader_ast.clone()),
                platform: None,
                shader_ast: Some(shader_ast),
            });
            self.shader_recompile_ids.push(shader_id);

            shader.shader_id.store(shader_id, Ordering::Relaxed);

            shader_id
        }
    }
}
