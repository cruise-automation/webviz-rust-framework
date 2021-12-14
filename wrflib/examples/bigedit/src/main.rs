// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// Not great but we do these comparisons all over the place..
#![allow(clippy::float_cmp)]
// Clippy TODO
#![allow(clippy::all)]

mod builder;
mod buildmanager;
mod codeicon;
mod colorpicker;
mod fieldworld;
mod fileeditor;
mod filepanel;
mod filetree;
mod homepage;
mod itemdisplay;
mod jseditor;
mod keyboard;
mod listanims;
mod loglist;
mod makepadapp;
mod makepadstorage;
mod makepadwindow;
mod mprstokenizer;
mod plaineditor;
mod rusteditor;
mod searchindex;
mod searchresults;
mod treeworld;
mod worldview;

use crate::makepadapp::MakepadApp;
use wrflib::*;
main_app!(MakepadApp);
