// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

fn main() {
    #[cfg(target_os = "linux")] // this is the *current* os, not the target triple
    {
        // Per https://kazlauskas.me/entries/writing-proper-buildrs-scripts.html
        if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "linux" {
            println!("cargo:rustc-link-lib=GLX");
        }
    }
}
