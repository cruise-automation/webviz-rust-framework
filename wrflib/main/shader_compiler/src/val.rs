// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::math::Vec4;
use crate::util::PrettyPrintedFloat;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Val {
    Bool(bool),
    Int(i32),
    Float(f32),
    Vec4(Vec4),
}

impl Val {
    pub(crate) fn to_bool(&self) -> Option<bool> {
        match *self {
            Val::Bool(val) => Some(val),
            _ => None,
        }
    }

    pub(crate) fn to_int(&self) -> Option<i32> {
        match *self {
            Val::Int(val) => Some(val),
            _ => None,
        }
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Val::Bool(val) => write!(f, "{}", val),
            Val::Int(val) => write!(f, "{}", val),
            Val::Float(v) => write!(f, "{}", PrettyPrintedFloat(v)),
            Val::Vec4(val) => write!(f, "{}", val),
        }
    }
}
