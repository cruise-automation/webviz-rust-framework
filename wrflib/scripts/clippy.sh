#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# Check for Clippy errors
# Rules defined here represent intentional project wide configuration. To see outstanding Clippy TODOs, search
# the codebase for `Clippy TODO`.

# cargo clippy --workspace --fix --allow-dirty --allow-staged --all-targets --
cargo clippy --workspace --all-targets -- \
    -D clippy::all \
    -A clippy::single_match \
    -A clippy::too_many_arguments \
    -A clippy::comparison_chain \
    -A clippy::branches_sharing_code \
