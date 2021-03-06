#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# --no-deps and individual package specification because otherwise some deps crash during rustdoc.
# BROWSER=echo and --open per https://github.com/rust-lang/cargo/issues/5562#issuecomment-887068135
# RUSTDOCFLAGS="-Dwarnings" so warnings are turned into errors.
RUSTDOCFLAGS="-Dwarnings" BROWSER=echo cargo doc --open --no-deps -p wrflib -p wrflib_components
