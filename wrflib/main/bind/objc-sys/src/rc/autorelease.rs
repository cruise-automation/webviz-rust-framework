// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use runtime::{objc_autoreleasePoolPop, objc_autoreleasePoolPush};
use std::os::raw::c_void;

// we use a struct to ensure that objc_autoreleasePoolPop during unwinding.
struct AutoReleaseHelper {
    context: *mut c_void,
}

impl AutoReleaseHelper {
    unsafe fn new() -> Self {
        AutoReleaseHelper { context: objc_autoreleasePoolPush() }
    }
}

impl Drop for AutoReleaseHelper {
    fn drop(&mut self) {
        unsafe { objc_autoreleasePoolPop(self.context) }
    }
}

/**
Execute `f` in the context of a new autorelease pool. The pool is drained
after the execution of `f` completes.

This corresponds to `@autoreleasepool` blocks in Objective-C and Swift.
*/
pub fn autoreleasepool<T, F: FnOnce() -> T>(f: F) -> T {
    let _context = unsafe { AutoReleaseHelper::new() };
    f()
}
