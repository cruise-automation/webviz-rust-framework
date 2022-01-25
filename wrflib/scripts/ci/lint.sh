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

# TODO(JP): Move to Dockerfile-ci?.
rustup component add rustfmt --toolchain nightly-2022-01-18-x86_64-unknown-linux-gnu

cargo fmt --all -- --check # checks formatting for all Rust files

wrflib/scripts/clippy.sh

# Make sure rustdoc works without warnings as well.
wrflib/scripts/build_rustdoc.sh
