#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}/../.."

# We build to a fresh target directory to make sure we don't have any stale files that we
# copy to the website.
rm -rf website_dev/

# Run the developer version of this script.
wrflib/scripts/build_website_dev.sh

# Copy over just the files that we actually want to ship.
rm -rf website/
mkdir website/
cp -R wrflib/website_root/* website/
cp -R website_dev/docs website/docs
mkdir website/target
cp -R website_dev/target/doc website/target/doc
cp -R website_dev/*.html website/
mkdir -p website/target/wasm32-unknown-unknown/release/
cp website_dev/target/wasm32-unknown-unknown/release/*.wasm website/target/wasm32-unknown-unknown/release/
mkdir -p website/wrflib/web/dist/
cp website_dev/wrflib/web/dist/* website/wrflib/web/dist/
mkdir -p website/wrflib/examples/example_bigedit/src/
cp website_dev/wrflib/examples/example_bigedit/src/treeworld.rs website/wrflib/examples/example_bigedit/src/treeworld.rs

echo 'Website generated for publishing! Open using `website/server.sh` or publish `website/`'
