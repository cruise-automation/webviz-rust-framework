// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// lets classify the types of things we can autogenerate
// empty UI file
// View entry
// Quad entry
// QuadwShader entry

use crate::filetree::*;
use wrflib::*;
use wrflib_widget::*;

pub struct FilePanel {
    pub file_tree: FileTree,
    pub new_file_btn: NormalButton,
}
/*
#[derive(Clone, PartialEq)]
pub enum FilePanelEvent {
    NewFile {name: String, template: String},
    Cancel,
    None,
}
*/
impl FilePanel {
    pub fn new() -> Self {
        Self { file_tree: FileTree::new(), new_file_btn: NormalButton::default() }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> FileTreeEvent {
        //self.new_file_btn.handle_button(cx, event);
        self.file_tree.handle(cx, event)
    }

    pub fn draw_tab(&mut self, _cx: &mut Cx) {
        //self.new_file_btn.draw_button(cx, "HELLO");
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.file_tree.draw(cx)
    }
}
