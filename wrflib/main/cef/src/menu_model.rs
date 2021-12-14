// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib_cef_sys::cef_menu_model_t;

use crate::ptr::RefCounterGuard;

#[derive(Clone)]
pub struct MenuModel {
    ptr: RefCounterGuard<cef_menu_model_t>,
}
impl MenuModel {
    pub(crate) fn from(ptr: *mut cef_menu_model_t, track_ref: bool) -> MenuModel {
        unsafe { MenuModel { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    pub fn clear(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().clear {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }
    // TODO: implement other methods
}
