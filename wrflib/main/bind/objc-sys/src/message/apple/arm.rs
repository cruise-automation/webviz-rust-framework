// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use std::any::{Any, TypeId};
use std::mem;

use runtime::Imp;

extern "C" {
    fn objc_msgSend();
    fn objc_msgSend_stret();

    fn objc_msgSendSuper();
    fn objc_msgSendSuper_stret();
}

pub fn msg_send_fn<R: Any>() -> Imp {
    // Double-word sized fundamental data types don't use stret,
    // but any composite type larger than 4 bytes does.
    // <http://infocenter.arm.com/help/topic/com.arm.doc.ihi0042e/IHI0042E_aapcs.pdf>

    let type_id = TypeId::of::<R>();
    if mem::size_of::<R>() <= 4
        || type_id == TypeId::of::<i64>()
        || type_id == TypeId::of::<u64>()
        || type_id == TypeId::of::<f64>()
    {
        objc_msgSend
    } else {
        objc_msgSend_stret
    }
}

pub fn msg_send_super_fn<R: Any>() -> Imp {
    let type_id = TypeId::of::<R>();
    if mem::size_of::<R>() <= 4
        || type_id == TypeId::of::<i64>()
        || type_id == TypeId::of::<u64>()
        || type_id == TypeId::of::<f64>()
    {
        objc_msgSendSuper
    } else {
        objc_msgSendSuper_stret
    }
}
