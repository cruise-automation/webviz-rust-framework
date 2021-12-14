// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::math::Vec4;
use crate::ty::Ty;
use crate::val::Val;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Lit {
    Bool(bool),
    Int(i32),
    Float(f32),
    Vec4(Vec4),
}

impl Lit {
    pub(crate) fn to_ty(self) -> Ty {
        match self {
            Lit::Bool(_) => Ty::Bool,
            Lit::Int(_) => Ty::Int,
            Lit::Float(_) => Ty::Float,
            Lit::Vec4(_) => Ty::Vec4,
        }
    }

    pub(crate) fn to_val(self) -> Val {
        match self {
            Lit::Bool(lit) => Val::Bool(lit),
            Lit::Int(lit) => Val::Int(lit as i32),
            Lit::Float(lit) => Val::Float(lit),
            Lit::Vec4(lit) => Val::Vec4(lit),
        }
    }
}

impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Lit::Bool(lit) => write!(f, "{}", lit),
            Lit::Int(lit) => write!(f, "{}", lit),
            Lit::Float(lit) => {
                if lit.abs().fract() < 0.00000001 {
                    write!(f, "{}.0", lit)
                } else {
                    write!(f, "{}", lit)
                }
            }
            Lit::Vec4(lit) => {
                write!(f, "vec4({},{},{},{})", lit.x, lit.y, lit.z, lit.w)
            }
        }
    }
}
