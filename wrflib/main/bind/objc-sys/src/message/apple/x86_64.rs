// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use std::mem;

use crate::runtime::Imp;

extern "C" {
    fn objc_msgSend();
    fn objc_msgSend_stret();

    fn objc_msgSendSuper();
    fn objc_msgSendSuper_stret();
}

pub fn msg_send_fn<R>() -> Imp {
    // If the size of an object is larger than two eightbytes, it has class MEMORY.
    // If the type has class MEMORY, then the caller provides space for the return
    // value and passes the address of this storage.
    // <http://people.freebsd.org/~obrien/amd64-elf-abi.pdf>

    if mem::size_of::<R>() <= 16 {
        objc_msgSend
    } else {
        objc_msgSend_stret
    }
}

pub fn msg_send_super_fn<R>() -> Imp {
    if mem::size_of::<R>() <= 16 {
        objc_msgSendSuper
    } else {
        objc_msgSendSuper_stret
    }
}
