// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use objc_exception;

use rc::StrongPtr;
use runtime::Object;

pub unsafe fn try<F, R>(closure: F) -> Result<R, StrongPtr>
        where F: FnOnce() -> R {
    objc_exception::try(closure).map_err(|exception| {
        StrongPtr::new(exception as *mut Object)
    })
}
