#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}"

TAG=$(git rev-parse HEAD | head -c 8)

docker tag exviz-mp-base-ci:$TAG janpaul123/exviz-mp-base-ci:latest
docker push janpaul123/exviz-mp-base-ci:latest
