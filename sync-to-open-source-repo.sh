#!/bin/bash

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}/.."

TEMP_REPO=__temp-open-source-wrflib__

# Some sanity checks.
if [[ $(git rev-parse --abbrev-ref HEAD) != "master" ]]; then
  read -p "Not on 'master' branch, continue? (y/N) " choice
  case "$choice" in
    y|Y ) echo "Continuing...";;
    * ) exit;;
  esac
fi

if [[ -n $(git status --porcelain) ]]; then
  read -p "Git tree is dirty, continue? (y/N) " choice
  case "$choice" in
    y|Y ) echo "Continuing...";;
    * ) exit;;
  esac
fi

# Remove potential existing directory (after unclean script exit).
rm -rf $TEMP_REPO

# Clone open source repo into new directory.
git clone git@github.com:cruise-automation/webviz-rust-framework.git $TEMP_REPO

# Sync up files.
rsync -av --delete --exclude .git --exclude target/ wrflib_open_source_root/ $TEMP_REPO/
rsync -av --delete --exclude .git --exclude target/ wrflib/ $TEMP_REPO/wrflib/
rsync -av rustfmt.toml $TEMP_REPO/rustfmt.toml
pushd $TEMP_REPO
  # Create a new branch (actually for now let's just commit directly to master)
  # git checkout -b update-$(date +%s)
  # Add everything.
  git add .
  # Commit! This should bring up an interactive editor so the user can write a commit message.
  git commit --verbose --template <(printf "Update from internal repo\n\nChangelog:\n-")
  # Push the new branch.
  git push --all -u
popd

# Clean up.
rm -rf $TEMP_REPO
