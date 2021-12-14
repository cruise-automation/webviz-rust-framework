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
cargo build --all-targets --workspace # builds everything in the workspace, including tests, etc
wrflib/scripts/build_wasm.sh --all-targets --workspace # same but for wasm
