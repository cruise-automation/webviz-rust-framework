// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use std::any::Any;

use super::{Message, MessageArguments, MessageError, Super};
use runtime::{Class, Object, Sel};

#[cfg(target_arch = "x86")]
#[path = "x86.rs"]
mod arch;
#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
mod arch;
#[cfg(target_arch = "arm")]
#[path = "arm.rs"]
mod arch;
#[cfg(target_arch = "aarch64")]
#[path = "arm64.rs"]
mod arch;

use self::arch::{msg_send_fn, msg_send_super_fn};

pub unsafe fn send_unverified<T, A, R>(obj: *const T, sel: Sel, args: A) -> Result<R, MessageError>
where
    T: Message,
    A: MessageArguments,
    R: Any,
{
    let receiver = obj as *mut T as *mut Object;
    let msg_send_fn = msg_send_fn::<R>();
    objc_try!({ A::invoke(msg_send_fn, receiver, sel, args) })
}

pub unsafe fn send_super_unverified<T, A, R>(obj: *const T, superclass: &Class, sel: Sel, args: A) -> Result<R, MessageError>
where
    T: Message,
    A: MessageArguments,
    R: Any,
{
    let sup = Super { receiver: obj as *mut T as *mut Object, superclass };
    let receiver = &sup as *const Super as *mut Object;
    let msg_send_fn = msg_send_super_fn::<R>();
    objc_try!({ A::invoke(msg_send_fn, receiver, sel, args) })
}
