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
cargo build --release # builds a standard release build for the current operating system
wrflib/scripts/build_wasm.sh --release # release build for wasm only
