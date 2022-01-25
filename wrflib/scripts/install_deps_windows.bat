rustup toolchain install nightly-2022-01-18
rustup target add wasm32-unknown-unknown
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-gnu
rustup component add rustfmt
rustup component add clippy
cargo install cargo-bundle mdbook
rustup component add rust-src

REM TODO(JP): auto-download CEF here... (from https://cef-builds.spotifycdn.com/index.html#windows64)
