// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use filetime::FileTime;
use regex::Regex;
use std::fs;
use std::io::{self, Read};
/// Use wasm-bindgen-threads-xform to transform our compiled WASM modules into
/// thread-safe implementations. Without this, we run into issues with shared
/// memory across WebWorkers.
///
/// TODO(Paras): This definitely adds a few seconds to our build, so we could consider
/// injecting this code directly ourselves and prevent the extra translation step.
///
/// This file is used only by scripts/build_wasm.sh. See usage there.

fn transform_wasm(path: String) -> anyhow::Result<()> {
    // Write to a new file and preserve the source file so that this process
    // is idempotent and dev environments can run this twice in a row.
    let dest = path.replace(".wasm", "-xform.wasm");

    // Skip if transformed WASM exists and original WASM unchanged
    if let Ok(dest_metadata) = fs::metadata(&dest) {
        let path_metadata = fs::metadata(&path).unwrap();
        let path_modified = FileTime::from_last_modification_time(&path_metadata);
        let dest_modified = FileTime::from_last_modification_time(&dest_metadata);
        if dest_modified > path_modified {
            return Ok(());
        }
    }

    let mut m = walrus::Module::from_file(&path)?;
    wasm_bindgen_threads_xform::Config::new().maximum_memory(u32::MAX).run(&mut m)?;

    std::fs::write(dest, m.emit_wasm())?;

    Ok(())
}

// TODO(Paras): The following code is depressingly ugly. I could not find a clean way
// to determine which files had been produced by the preceding cargo output, so I
// messily parse it here.
fn main() -> anyhow::Result<()> {
    let mut data = String::new();
    io::stdin().read_to_string(&mut data)?;
    // Find the part of the JSON with `"filenames":[foo.wasm, bar.wasm]`."
    let re = Regex::new(r#"filenames":\[([^\]]*)"#).unwrap();
    for cap in re.captures_iter(&data) {
        for f in cap[1].split(',') {
            // When running `--workspace  --all-targets`, ignore dependencies and this file itself.
            // This is a pretty fragile way to ignore dependencies, and will break if any other part of the
            // path contains 'deps' or 'wrflib_wasm_thread_xform'.
            if f.ends_with(".wasm\"") && !f.contains("/deps/") && !f.contains("wrflib_wasm_thread_xform") {
                let filename = f.replace("\"", "");
                transform_wasm(filename)?;
            }
        }
    }

    Ok(())
}
