// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use runtime::Imp;

extern "C" {
    fn objc_msgSend();

    fn objc_msgSendSuper();
}

pub fn msg_send_fn<R>() -> Imp {
    // stret is not even available in arm64.
    // <https://twitter.com/gparker/status/378079715824660480>

    objc_msgSend
}

pub fn msg_send_super_fn<R>() -> Imp {
    objc_msgSendSuper
}
