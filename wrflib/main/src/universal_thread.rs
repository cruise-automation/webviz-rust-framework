// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! Version of [`std::thread`] that also works in WebAssembly.

use std::thread;
use std::time::Duration;
/// See [`Thread`].
struct UniversalThread();

/// Encapsulates the public thread API that has been tested on native and WASM,
/// as well as environment specific implementation. All methods from here
/// will get exposed at the module level as well lower in this file.
trait Thread {
    /// Run function in a non-blocking thread.
    ///
    /// Check out the `multithread_example` for sample usage.
    ///
    /// TODO(Paras): Implement join handles. When we do, we can use the same
    /// function signature here as [`std::thread::spawn`].
    fn spawn(f: impl FnOnce() + Send + 'static);

    /// See [`std::thread::sleep`].
    fn sleep(duration: Duration);
}

#[cfg(not(target_arch = "wasm32"))]
impl Thread for UniversalThread {
    /// See [`Thread::spawn`].
    fn spawn(f: impl FnOnce() + Send + 'static) {
        thread::spawn(f);
    }

    /// See [`Thread::sleep`].
    fn sleep(dur: Duration) {
        thread::sleep(dur);
    }
}

#[cfg(target_arch = "wasm32")]
struct WorkerContext {
    func: Box<dyn FnOnce() + Send>,
}

#[cfg(target_arch = "wasm32")]
impl Thread for UniversalThread {
    /// See [`Thread::spawn`].
    fn spawn(f: impl FnOnce() + Send + 'static) {
        let context = Box::into_raw(Box::new(WorkerContext { func: Box::new(f) })) as usize;

        unsafe {
            threadSpawn(context as u64);
        }
    }

    /// See [`Thread::sleep`].
    fn sleep(dur: Duration) {
        thread::sleep(dur);
    }
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn threadSpawn(context: u64);
}

#[cfg(target_arch = "wasm32")]
#[export_name = "runFunctionPointer"]
unsafe extern "C" fn fn_to_run_in_worker(ctx_ptr: u64) {
    let ctx = Box::from_raw(ctx_ptr as *mut WorkerContext);
    (ctx.func)();
}

/// Version of [`std::thread::spawn`] that also works in WebAssembly.
///
/// See also [`Thread::spawn`].
pub fn spawn(f: impl FnOnce() + Send + 'static) {
    UniversalThread::spawn(f);
}

pub fn sleep(dur: Duration) {
    UniversalThread::sleep(dur);
}
