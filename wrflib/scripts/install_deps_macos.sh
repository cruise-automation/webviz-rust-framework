#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}"

if type xcode-select >&- && xpath=$( xcode-select --print-path ) && test -d "${xpath}" && test -x "${xpath}" ; then
    echo "Xcode command line tools are already installed.";
else
    xcode-select --install
fi

# Used by `kill-descendants-on-exit.bash`.
brew install pstree

rustup toolchain install nightly-2021-08-03
rustup target add wasm32-unknown-unknown
rustup component add rustfmt
rustup component add clippy
cargo install cargo-bundle
rustup component add rust-src

rm -rf ../main/bind/cef-sys/deps/cef_binary_*
mkdir -p ../main/bind/cef-sys/deps
pushd ../main/bind/cef-sys/deps
    # Check out `wrflib/main/bind/cef-sys/README.md` file for notes on the current CEF/Chromium version.
    curl "https://cef-builds.spotifycdn.com/cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64.tar.bz2" | tar x -
popd
