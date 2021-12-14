// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib_shader_compiler::math::Rect;

/// Enum to encapsulate various events that happens during draw call
#[derive(Clone)]
pub enum DebugLog {
    /// For cases when cx.end_turtle() is getting called
    EndTurtle { rect: Rect },
}