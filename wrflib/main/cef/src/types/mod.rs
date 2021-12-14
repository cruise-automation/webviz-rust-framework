// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

pub mod color;
mod point;
mod range;
mod rect;
mod size;
pub mod string;

pub use color::*;
pub use point::*;
pub use range::*;
pub use rect::*;
pub use size::*;

pub type LogSeverity = wrflib_cef_sys::cef_log_severity_t;
pub type PaintElementType = wrflib_cef_sys::cef_paint_element_type_t;
pub type TextInputMode = wrflib_cef_sys::cef_text_input_mode_t;
pub type DragOperationsMask = wrflib_cef_sys::cef_drag_operations_mask_t;
pub type ThreadId = wrflib_cef_sys::cef_thread_id_t;
pub type ProcessId = wrflib_cef_sys::cef_process_id_t;
