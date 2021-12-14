# 🐢⚡ Webviz Rust Framework (Wrf)

Wrf is a fast, cross-platform (web+native), GPU-based application framework, written in Rust.

We're a small team at [Cruise](https://getcruise.com/) building an internal application and this framework at the same time. It's in early stages of development, so don't expect too much yet!

The current supported workflow is to clone this repo and then add your own package to the root Cargo.toml.

## Development

* [Install Rust](https://www.rust-lang.org/tools/install)
* Run the appropriate dependency installation script in `wrflib/scripts/install_deps_*`
* Native build of Bigedit (a modified version of the original Makepad editor)
  * Run debug mode: `cargo run -p bigedit`
  * Run production mode: `cargo run -p bigedit --release`
* WebAssembly build
  * Run a webserver in the root directory using `wrflib/scripts/server.py`
  * Run debug mode: `wrflib/scripts/build_wasm.sh -p bigedit` and go to http://localhost:5000/wrflib/examples/bigedit/?debug=true
  * Run production mode: `wrflib/scripts/build_wasm.sh -p bigedit --release` and go to http://localhost:5000/wrflib/examples/bigedit/
* Continuous Integration (CI)
  * We have a bunch of CI scripts in `wrflib/scripts/ci` but they're not set up for this Github repository yet.

## Makepad repo

Originally we forked the [Makepad Framework](https://github.com/makepad/makepad), but we've heavily modified the codebase since then, and are pursuing our own vision. The editor itself is still around, but has also been heavily modified and has way fewer features; it can be found in wrflib/examples/bigedit.

## License

Wrf is primarily distributed under the terms of both the MIT license and the Apache License (version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
