#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

# TODO(JP): The path where we put CEF originally is a bit funky, so it would be nice to clean
# that up at some point. Still, this is better than downloading it from the internet again..
cp -r /main/bind/cef-sys/deps/* wrflib/main/bind/cef-sys/deps/
