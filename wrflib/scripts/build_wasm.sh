#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# TODO(JP): Move this to a Rust command, e.g. `cargo zaplib build` or so. That way you can `cargo install` this script.
# TODO(JP): Safari doesn't support SIMD yet, so for now you have to manually remove the +simd128 flag if you want to support Safari..
# max-memory=4294967296 = 65536 (max pages) * 65536 (page size)
# Export __stack_pointer to get the shadow stack pointer; see e.g.:
# - https://github.com/rustwasm/wasm-bindgen/blob/ac87c8215bdd28d6aa0e12705996238a78227f8c/crates/wasm-conventions/src/lib.rs#L36
# - https://github.com/WebAssembly/tool-conventions/blob/main/Linking.md#merging-global-sections
build_command='RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals,+simd128 -C link-arg=--max-memory=4294967296 -C link-arg=--export=__stack_pointer" cargo build --target=wasm32-unknown-unknown -Zbuild-std=std,panic_abort "$@"'

# First build normally, for human readable compile errors
echo "    Running cargo build $@"
eval "$build_command"
