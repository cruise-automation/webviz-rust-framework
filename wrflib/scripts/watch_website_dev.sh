#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}/../.."

cargo install cargo-watch

# For some reason mdbook touches image files on compilation, so we ignore those..
cargo watch --why --ignore 'docs/src/img/*' --watch wrflib/ --shell wrflib/scripts/build_website_dev.sh
