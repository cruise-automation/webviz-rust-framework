#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}"

pushd ../main/bind/cef-sys/deps
    curl "https://cef-builds.spotifycdn.com/cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64_release_symbols.tar.bz2" | tar x -
    mv cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64_release_symbols/Chromium\ Embedded\ Framework.dSYM cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64/Release/Chromium\ Embedded\ Framework.framework/Chromium\ Embedded\ Framework.dSYM
    rm -r cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64_release_symbols

    curl "https://cef-builds.spotifycdn.com/cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64_debug_symbols.tar.bz2" | tar x -
    mv cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64_debug_symbols/Chromium\ Embedded\ Framework.dSYM cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64/Debug/Chromium\ Embedded\ Framework.framework/Chromium\ Embedded\ Framework.dSYM
    rm -r cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64_debug_symbols
popd
