// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib_cef_sys::cef_request_t;

use crate::{ptr::RefCounterGuard, CefString};

pub(crate) struct Request {
    ptr: RefCounterGuard<cef_request_t>,
}

impl Request {
    pub(crate) fn from(ptr: *mut cef_request_t, track_ref: bool) -> Request {
        unsafe { Request { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    pub(crate) unsafe fn get_url(&self) -> Option<String> {
        if let Some(func) = self.ptr.as_ref().get_url {
            let ptr = func(self.ptr.get());
            let res = CefString::from_cef(ptr);
            wrflib_cef_sys::cef_string_userfree_utf16_free(ptr);
            Some(res.to_string())
        } else {
            None
        }
    }
}
