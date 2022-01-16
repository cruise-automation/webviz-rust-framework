#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

wrflib/scripts/ci/common.sh

export RUSTFLAGS="-D warnings"

# Run a check (not a build) for the various target triples.
cargo check --release --workspace --target x86_64-unknown-linux-gnu --exclude tutorial_js_rust_bridge
cargo check --release --workspace --target wasm32-unknown-unknown
# `--no-default-features` is to disable TLS since it breaks cross-compilation
# `--exclude wrflib_cef(_sys)` and `test_suite` since we currently don't support cross-compiling with CEF.
cargo check --release --workspace --target x86_64-apple-darwin --no-default-features --exclude wrflib_cef --exclude wrflib_cef_sys --exclude test_suite --exclude tutorial_js_rust_bridge
cargo check --release --workspace --target x86_64-pc-windows-msvc --no-default-features --exclude wrflib_cef --exclude wrflib_cef_sys --exclude test_suite --exclude tutorial_js_rust_bridge
cargo check --release --workspace --target x86_64-pc-windows-gnu --no-default-features --exclude wrflib_cef --exclude wrflib_cef_sys --exclude test_suite --exclude tutorial_js_rust_bridge
