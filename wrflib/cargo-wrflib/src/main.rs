// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use std::process::Command;

use clap::{App, AppSettings, Arg};

fn main() {
    let matches = App::new("WRFlib Command Line Tool")
        .setting(AppSettings::ArgRequiredElseHelp)
        .about(env!["CARGO_PKG_DESCRIPTION"])
        .version(env!("CARGO_PKG_VERSION"))
        // When running as a cargo command, the second argument is the name of the program
        // and we want to ignore it when displaying help
        .arg(Arg::new("crate-version").hide(true))
        .arg(Arg::new("install-deps").short('I').long("install-deps").takes_value(false).help("Install development dependencies"))
        .subcommand(
            App::new("build")
                .arg(
                    Arg::new("release")
                        .short('R')
                        .long("release")
                        .takes_value(false)
                        .help("Build artifacts in release mode, with optimizations"),
                )
                .arg(Arg::new("simd128").long("simd128").takes_value(false).help("Use 128-bit SIMD instruction set for WASM")),
        )
        .get_matches();

    if let Some(cmd) = matches.subcommand_matches("build") {
        build(BuildOpts { release: cmd.is_present("release"), use_simd128: cmd.is_present("simd128") });
    } else if matches.is_present("install-deps") {
        install_deps();
    }
}

fn install_deps() {
    unimplemented!();
}

#[derive(Default, Debug)]
struct BuildOpts {
    release: bool,
    use_simd128: bool,
}

fn build(opts: BuildOpts) {
    println!("    Running cargo build");

    let mut args = vec!["+nightly-2021-08-03", "build", "--target=wasm32-unknown-unknown", "-Zbuild-std=std,panic_abort"];
    if opts.release {
        args.push("--release");
    }

    let rust_flags = {
        let mut flags = vec![];
        if opts.use_simd128 {
            flags.push("-C target-feature=+atomics,+bulk-memory,+mutable-globals,+simd128");
        } else {
            flags.push("-C target-feature=+atomics,+bulk-memory,+mutable-globals");
        }
        flags.push("-C link-arg=--max-memory=4294967296");
        flags.push("-C link-arg=--export=__stack_pointer");

        flags.join(" ")
    };

    let out = Command::new("cargo").env("RUSTFLAGS", &rust_flags).args(args).output().expect("Failed to execute command");
    println!("{}", std::str::from_utf8(&out.stdout).ok().unwrap());
    println!("{}", std::str::from_utf8(&out.stderr).ok().unwrap());
}
