# Getting Started

First let's install some dependencies:
* [Install Rust](https://www.rust-lang.org/tools/install)
* Clone the repo: `git clone git@github.com:cruise-automation/webviz-rust-framework.git`
* Navigate to the repo: `cd webviz-rust-framework`
* Run the appropriate dependency installation script:
  * Mac OS X: `wrflib/scripts/install_deps_macos.sh`
  * Windows: `wrflib/scripts/install_deps_windows.bat`
  * Linux: `wrflib/scripts/install_deps_linux.sh`

Now you're ready to run a simple example natively. Here are some fun ones to play with:
* `cargo run -p example_single_button`
* `cargo run -p example_charts`
* `cargo run -p example_text`
* `cargo run -p example_bigedit` (heavy; best to do a release build; see below)

For a more performant build, add the `--release` flag, e.g.:
* `cargo run -p example_single_button --release`

Of course, Wrflib is primarily a framework for WebAssembly, so let's run these examples in a browser:
* Download the latest version of a modern browser, like [Chrome](https://www.google.com/chrome/).
* In a separate terminal window, run a basic server: `wrflib/scripts/server.py` (Note that this still requires Python 2).
* In another separate terminal window, start yarn to build the Wrflib javascript files:
  * `cd wrflib/web && yarn && yarn watch`
* Build one of the examples using the `build_wasm.sh` script, e.g.:
  * `wrflib/scripts/build_wasm.sh -p example_single_button`
* Navigate your browser to:
  * [`http://localhost:3000/wrflib/examples/example_single_button`](http://localhost:3000/wrflib/examples/example_single_button)
* Again, for a more performant build, add the `--release` flag, e.g.:
  * `wrflib/scripts/build_wasm.sh -p example_single_button --release`
* With a release build, add a `?release` flag to the URL:
  * [`http://localhost:3000/wrflib/examples/example_single_button/?release`](http://localhost:3000/wrflib/examples/example_single_button/?release)

Feel free to check out the `examples` directory for more examples to play with!

To view automatically generated API documentation, run:
* `wrflib/scripts/build_rustdoc.sh`

If you're wondering what to do next, here are some options:
* Set up your [tooling](./basic_tooling.md).
* Dive into some tutorials.
* Look at the code for one of the examples (`example_single_button` is a great simple one to start with) and try to modify it.
