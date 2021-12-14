// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use std::cell::UnsafeCell;
use std::ptr;

use super::StrongPtr;
use runtime::{self, Object};

// Our pointer must have the same address even if we are moved, so Box it.
// Although loading the WeakPtr may modify the pointer, it is thread safe,
// so we must use an UnsafeCell to get a *mut without self being mutable.

/// A pointer that weakly references an object, allowing to safely check
/// whether it has been deallocated.
pub struct WeakPtr(Box<UnsafeCell<*mut Object>>);

impl WeakPtr {
    /// Constructs a `WeakPtr` to the given object.
    /// # Safety
    /// Unsafe because the caller must ensure the given object pointer is valid.
    pub unsafe fn new(obj: *mut Object) -> Self {
        let ptr = Box::new(UnsafeCell::new(ptr::null_mut()));
        runtime::objc_initWeak(ptr.get(), obj);
        WeakPtr(ptr)
    }

    /// Loads the object self points to, returning a `StrongPtr`.
    /// If the object has been deallocated, the returned pointer will be null.
    pub fn load(&self) -> StrongPtr {
        unsafe {
            let ptr = runtime::objc_loadWeakRetained(self.0.get());
            StrongPtr::new(ptr)
        }
    }
}

impl Drop for WeakPtr {
    fn drop(&mut self) {
        unsafe {
            runtime::objc_destroyWeak(self.0.get());
        }
    }
}

impl Clone for WeakPtr {
    fn clone(&self) -> Self {
        let ptr = Box::new(UnsafeCell::new(ptr::null_mut()));
        unsafe {
            runtime::objc_copyWeak(ptr.get(), self.0.get());
        }
        WeakPtr(ptr)
    }
}
