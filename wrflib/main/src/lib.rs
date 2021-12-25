//! 🐢⚡ This crate is the core of Wrf. It contains all the
//! fundamental rendering primitives.
//!
//! Internally it depends on [`wrflib_shader_compiler`] and [`wrflib_vector`],
//! for shader compilation and vector graphics (mostly for fonts) respectively.
//!
//! If you need to use higher-level widgets, use `wrflib_widget`.

// Necessary in cx_xlib
#![allow(temporary_cstring_as_ptr)]
// Not great but we do these comparisons all over the place..
#![allow(clippy::float_cmp)]
// We want to use links to private fields, since we use `--document-private-items`.
#![allow(rustdoc::private_intra_doc_links)]
// For using [`std::alloc::set_alloc_error_hook`].
#![cfg_attr(target_arch = "wasm32", feature(alloc_error_hook))]
// For using [`core::arch::wasm32`].
#![cfg_attr(target_arch = "wasm32", feature(stdsimd))]

#[macro_use]
mod macros;

mod cx;

#[cfg(any(target_os = "linux"))]
mod cx_linux;
#[cfg(target_os = "linux")]
mod cx_opengl;
#[cfg(target_os = "linux")]
mod cx_xlib;

#[cfg(any(target_os = "macos"))]
mod cx_apple;
#[cfg(target_os = "macos")]
mod cx_cocoa;
#[cfg(any(target_os = "macos"))]
mod cx_macos;
#[cfg(target_os = "macos")]
mod cx_metal;

#[cfg(target_os = "windows")]
mod cx_dx11;
#[cfg(target_os = "windows")]
mod cx_win32;
#[cfg(any(target_os = "windows"))]
mod cx_windows;

#[cfg(target_arch = "wasm32")]
mod cx_wasm32;
#[cfg(target_arch = "wasm32")]
mod cx_webgl;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod cx_desktop;

#[cfg(feature = "cef")]
mod cef_browser;

#[cfg(feature = "cef")]
mod cef_utils;

#[cfg(any(target_arch = "wasm32", feature = "cef"))]
mod zerde;

#[cfg(any(target_arch = "wasm32", feature = "cef"))]
mod cx_web;

mod cx_app_load;

mod animator;
mod area;
pub mod byte_extract;
mod colors;
mod component_base;
mod cursor;
pub mod debug_log;
mod debugger;
mod draw_tree;
mod events;
mod fonts;
mod geometry;
mod hash;
mod layout;
mod pass;
mod profile;
mod read_seek;
mod shader;
mod texture;
mod turtle;
pub mod universal_file;
pub mod universal_http_stream;
mod universal_instant;
pub mod universal_rand;
pub mod universal_thread;
mod window;

mod cube_ins;
mod image_ins;
mod menu;
mod quad_ins;
mod std_shader;
mod text_ins;

pub use crate::cube_ins::*;
pub use crate::cx::*;
pub use crate::debugger::*;
pub use crate::image_ins::*;
pub use crate::quad_ins::*;
pub use crate::std_shader::*;
pub use crate::text_ins::*;
