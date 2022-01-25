# API Overview

This is an overview of the different APIs for communicating between JavaScript and Rust.

The [Wrflib package](https://www.npmjs.com/package/wrflib) on npm has two entrypoints:
1. `wrflib_runtime.js`: the main runtime, to be used on the browser's main thread.
2. `wrflib_worker_runtime.js`: the Web Worker runtime, for use in your workers.

The APIs between these runtimes is mostly the same, but there are some small differences which we will note.

As noted in the [Introduction](./introduction.md), we also have a highly experimental feature of including Chromium in the native Mac OS X build, using [CEF](https://bitbucket.org/chromiumembedded/cef/src/master/). This gets enabled when compiling wrflib with the `cef` [feature](https://doc.rust-lang.org/cargo/reference/features.html). Generally this is not recommended to use in production yet, but we'll still note its level of support for the different APIs.
