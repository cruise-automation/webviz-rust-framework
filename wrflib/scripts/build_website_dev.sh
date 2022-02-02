#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}/../.."

# For the development version we don't use a a fresh target directory every time since we want to
# make it cheap to run this script (e.g. for `watch_website_dev.sh`).

mkdir -p website_dev/

cp -R wrflib/website_root/* website_dev/

cargo install mdbook
mdbook build wrflib/docs --dest-dir ../../website_dev/docs/

# --no-deps and individual package specification because otherwise some deps crash during rustdoc.
# RUSTDOCFLAGS="-Dwarnings" so warnings are turned into errors.
CARGO_TARGET_DIR="website_dev/target" RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps -p wrflib -p wrflib_components

mkdir -p website_dev/wrflib/examples/example_bigedit/src/
cp wrflib/examples/example_bigedit/src/treeworld.rs website_dev/wrflib/examples/example_bigedit/src/treeworld.rs

cp wrflib/examples/example_bigedit/index.html website_dev/example_bigedit.html
cp wrflib/examples/example_charts/index.html website_dev/example_charts.html
cp wrflib/examples/example_lightning/index.html website_dev/example_lightning.html
cp wrflib/examples/example_lots_of_buttons/index.html website_dev/example_lots_of_buttons.html
cp wrflib/examples/example_shader/index.html website_dev/example_shader.html
cp wrflib/examples/example_single_button/index.html website_dev/example_single_button.html
cp wrflib/examples/example_text/index.html website_dev/example_text.html

CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-wrflib -- build -p example_bigedit --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-wrflib -- build -p example_charts --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-wrflib -- build -p example_lightning --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-wrflib -- build -p example_lots_of_buttons --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-wrflib -- build -p example_shader --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-wrflib -- build -p example_single_button --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-wrflib -- build -p example_text --release

pushd wrflib/web/
    yarn
    # TODO(JP): This takes quite long! Look into caching this step.
    yarn build
popd
mkdir -p website_dev/wrflib/web/dist/
cp -R wrflib/web/dist/* website_dev/wrflib/web/dist/

echo 'Website generated for development! Open using `website_dev/server.sh`'
