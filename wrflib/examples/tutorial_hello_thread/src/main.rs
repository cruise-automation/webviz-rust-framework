// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use std::io::Read;

use wrflib::*;

#[derive(Default)]
struct App {
    window: Window,
}

impl App {
    fn new(_cx: &mut Cx) -> Self {
        Self { window: Window { create_add_drop_target_for_app_open_files: true, ..Window::default() }, ..Self::default() }
    }

    fn handle(&mut self, _cx: &mut Cx, event: &mut Event) {
        match event {
            Event::AppOpenFiles(aof) => {
                // Get a copy of the file handle for use in the thread.
                let mut file = aof.user_files[0].file.clone();

                universal_thread::spawn(move || {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();
                    log!("Contents of dropped file: {contents}");
                });
            }
            _ => {}
        }
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.window.end_window(cx);
    }
}

main_app!(App);
