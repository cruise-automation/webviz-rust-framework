// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! Compiles shaders using our own shader language, and outputs it into
//! various target shader languages.
//!
//! For internal Wrf use, unless you know what you're doing.

// We want to use links to private fields, since we use `--document-private-items`.
#![allow(rustdoc::private_intra_doc_links)]
// Clippy TODO
#![warn(clippy::all)]

mod analyse;
mod builtin;
mod const_eval;
mod const_gather;
mod dep_analyse;
mod detok;
mod env;
mod error;
mod generate;
pub mod generate_glsl;
pub mod generate_hlsl;
pub mod generate_metal;
pub mod generate_shader_ast;
mod ident;
mod lex;
mod lhs_check;
mod lit;
pub mod math;
mod shaderast;
mod shaderparser;
mod span;
mod swizzle;
mod token;
pub mod ty;
mod ty_check;
mod util;
mod val;

pub use shaderast::{Decl, ShaderAst};
