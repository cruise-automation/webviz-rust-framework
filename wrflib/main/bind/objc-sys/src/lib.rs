// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// doesn't work at all in wasm, so let's just not bother..
#![cfg(not(target_arch = "wasm32"))]
// Clippy TODO
#![allow(clippy::all)]

mod malloc_buf;

pub use crate::encode::{Encode, EncodeArguments, Encoding};
pub use crate::message::{Message, MessageArguments, MessageError};

pub use crate::message::send_message as __send_message;
pub use crate::message::send_super_message as __send_super_message;

#[macro_use]
mod macros;

pub mod declare;
mod encode;
mod message;
pub mod rc;
pub mod runtime;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
extern crate libc;
