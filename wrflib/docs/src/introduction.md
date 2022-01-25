# Introduction

**Wrflib** is an open-source library for speeding up web applications using Rust and WebAssembly. It lets you write high-performance code in Rust, alongside your existing JavaScript code, using simple APIs.

The goal of Wrflib is to make it easy to build performance-intensive applications in the browser. While it is possible to make JavaScript run fast, over time it may become hard to manage lots of optimizations. In Rust you tend to need way fewer optimizations to get to similar or even higher levels of performance, allowing you to focus on actually building stuff.

The idea is to start with your existing web-based codebase, and incrementally move pieces of code over to Wrflib:
* You might start with a small computation;
* then port some 2d/3d rendering;
* then move over some UI elements;
* and so on.

Over time, you could port your entire codebase over to Rust, or you might keep JavaScript and Rust code side-by-side.

## Structure

Wrflib roughly consists of these parts:
1. **Basic APIs.** A "standard library" for WebAssembly: console logging, low-level multithreading, HTTP requests, file reading, and so on.
2. **JS-Rust bridge.** Communicating data between JS and Rust.
3. **Rendering.** Low-level GPU-based 2d and 3d rendering APIs, and eventing.
4. **UI.** UI components, layout engine, animation.

Current development is mostly focused on 1-3, and at this point we recommend to keep using JavaScript/CSS for UI elements. But in the future we aim to support building entire applications fully within Wrflib.

The focus of Wrflib is on WebAssembly, but it also runs natively on various systems. This is useful while developing and testing components in isolation, comparable to using [Storybook](https://storybook.js.org/).

Wrflib runs on the following platforms:
1. **WebAssembly / WebGL.** Tested on recent versions of Chrome, Firefox, Edge, and Safari — though there are some known issues.
2. **Mac OS X / Metal.** Tested on 11.6 Big Sur (on Intel mostly).
3. **Windows / DirectX 11.** Not well supported; some APIs missing; but should run.
4. **Linux / OpenGL.** Not well supported; some APIs missing; but should run.

There is also a highly experimental feature where we embed a [Chromium](https://en.wikipedia.org/wiki/Chromium_(web_browser)) instance in a desktop build. This is similar to running Rust code alongside JavaScript in a browser using WebAssembly, except that your Rust code runs completely natively instead of in WebAssembly. Rendering is also done natively instead of using WebGL. This is generally more performant, and makes it easier to attach debuggers and profilers. We do not recommend using this in production yet, but it can be useful for debugging.

## License

Wrflib is primarily distributed under the terms of both the MIT license and the Apache License (version 2.0).

See `LICENSE-APACHE` and `LICENSE-MIT` in the root of the repo for details.
