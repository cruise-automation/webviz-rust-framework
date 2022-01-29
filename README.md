# 🐢⚡ Webviz Rust Framework (Wrflib)

Wrflib is a fast, cross-platform (web+native), GPU-based application framework, written in Rust.

We're a small team at [Cruise](https://getcruise.com/) building an internal application and this framework at the same time. It's in early stages of development, so don't expect too much yet!

The current supported workflow is to clone this repo and then add your own package to the root Cargo.toml.

# Development
## Installing
* [Install Rust](https://www.rust-lang.org/tools/install)
* Install cargo extension for Wrflib `cargo install cargo-wrflib`
* Run the dependency installation using Wrflib cargo tool `cargo wrflib install-deps`

## Cargo extension
Wrflib provides a cargo extension that can be used to perform different tasks. The extension can be installed like this: `cargo install cargo-wrflib`

Optionally, you can use the tool directly like so `cargo run -p cargo-wrflib`, as any other project.

## Docs

* [Install Rust](https://www.rust-lang.org/tools/install)
* Run `cargo install mdbook && mdbook watch wrflib/docs --open` to view the full set of docs.
* `wrflib/scripts/build_rustdoc.sh` is useful for automatically generated API documentation. It prints a URL that you can open in a browser.

## Makepad repo

Originally we forked the [Makepad Framework](https://github.com/makepad/makepad), but we've heavily modified the codebase since then, and are pursuing our own vision. The editor itself is still around, but has also been heavily modified and has way fewer features; it can be found in wrflib/examples/example_bigedit.

## License

Wrflib is primarily distributed under the terms of both the MIT license and the Apache License (version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
