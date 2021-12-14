#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

build_command='RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals,+simd128" cargo build --target=wasm32-unknown-unknown -Zbuild-std=std,panic_abort "$@"'

# First build normally, for human readable compile errors
echo "    Running cargo build $@"
eval "time $build_command --quiet"

# Build again with JSON output (results should be cached from previous run)
output=$(eval $build_command --message-format json)

# Transform generated wasm into thread safe code
echo "    Transforming WASM"
echo $output | WASM_BINDGEN_THREADS=1 cargo run --quiet -p wrflib_wasm_thread_xform
echo "    Finished transforming WASM"
