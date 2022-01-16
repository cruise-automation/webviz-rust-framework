// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! WebAssembly platform-specific entry point.

use crate::cx_web::*;
use crate::universal_file::UniversalFile;
use crate::zerde::*;
use crate::*;
use std::alloc;
use std::collections::{BTreeSet, HashMap};
use std::mem;
use std::ptr;
use std::sync::{Arc, RwLock};

impl Cx {
    /// Initialize global error handlers.
    pub fn init_error_handlers() {
        std::alloc::set_alloc_error_hook(|layout: std::alloc::Layout| {
            console_error("Allocation error! Printing the layout on the next line...");
            // Printing this separately, since it will do an allocation itself and so might fail!
            console_error(&format!("Allocation layout: {:?}", layout));
        });
        std::panic::set_hook(Box::new(|info: &std::panic::PanicInfo| {
            console_error(&format!("Panic: {}", info.to_string()));
        }));
    }

    pub fn process_wasm_events<F>(&mut self, msg: u64, mut event_handler: F) -> u64
    where
        F: FnMut(&mut Cx, &mut Event),
    {
        self.event_handler =
            Some(&mut event_handler as *const dyn FnMut(&mut Cx, &mut Event) as *mut dyn FnMut(&mut Cx, &mut Event));
        let ret = self.event_loop_core(msg);
        self.event_handler = None;
        ret
    }

    /// Incoming Zerde buffer. There is no other entrypoint to general rust codeflow than this function,
    /// only allocators and init. Note that we do have other outgoing functions for synchronous
    /// operations.
    fn event_loop_core(&mut self, msg: u64) -> u64 {
        if self.platform.is_initialized == false {
            self.platform.is_initialized = true;
            for _i in 0..10 {
                self.platform.fingers_down.push(false);
            }
        }

        let mut zerde_parser = ZerdeParser::from(msg);
        self.last_event_time = zerde_parser.parse_f64();
        let mut is_animation_frame = false;
        loop {
            let msg_type = zerde_parser.parse_u32();
            match msg_type {
                0 => {
                    // end
                    break;
                }
                1 => {
                    // fetch_deps
                    self.platform_type = PlatformType::Web {
                        port: zerde_parser.parse_u32() as u16,
                        protocol: zerde_parser.parse_string(),
                        hostname: zerde_parser.parse_string(),
                        pathname: zerde_parser.parse_string(),
                        search: zerde_parser.parse_string(),
                        hash: zerde_parser.parse_string(),
                    };
                    // send the UI our deps
                    // TODO(Paras): Use byte arrays instead (like in desktop targets) and remove
                    // the entire load_deps flow. We tried this once, but gave up in the short term
                    // due to race conditions in WASM that are only solved by some time delay between
                    // app init and font loading.
                    let mut load_deps = Vec::<String>::new();

                    for filename in FONT_FILENAMES {
                        load_deps.push(filename.to_string());
                    }
                    // other textures, things
                    self.platform.zerde_eventloop_msgs.load_deps(load_deps);
                }
                2 => {
                    // deps_loaded
                    let len = zerde_parser.parse_u32();
                    let mut write_fonts_data = self.fonts_data.write().unwrap();
                    write_fonts_data.fonts.resize(FONT_FILENAMES.len(), CxFont::default());
                    for _ in 0..len {
                        let dep_path = zerde_parser.parse_string();
                        let vec_rec = zerde_parser.parse_vec_ptr();
                        // check if its a font
                        for (font_id, filename) in FONT_FILENAMES.iter().enumerate() {
                            let filename_str = filename.to_string();
                            if filename_str == dep_path {
                                let mut cxfont = &mut write_fonts_data.fonts[font_id];
                                // load it
                                if cxfont.load_from_ttf_bytes(&vec_rec).is_err() {
                                    println!("Error loading font {} ", dep_path);
                                } else {
                                    cxfont.file = filename_str;
                                }
                                break;
                            }
                        }
                    }
                }
                3 => {
                    // init
                    self.platform.window_geom = WindowGeom {
                        is_fullscreen: false,
                        is_topmost: false,
                        inner_size: Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() },
                        dpi_factor: zerde_parser.parse_f32(),
                        outer_size: Vec2 { x: 0., y: 0. },
                        position: Vec2 { x: 0., y: 0. },
                        xr_is_presenting: false,
                        xr_can_present: zerde_parser.parse_u32() > 0,
                        can_fullscreen: zerde_parser.parse_u32() > 0,
                    };
                    self.default_dpi_factor = self.platform.window_geom.dpi_factor;
                    assert!(self.default_dpi_factor > 0.0);

                    if self.windows.len() > 0 {
                        self.windows[0].window_geom = self.platform.window_geom.clone();
                    }

                    self.wasm_event_handler(Event::Construct);

                    self.request_draw();
                }
                4 => {
                    // resize
                    let old_geom = self.platform.window_geom.clone();
                    self.platform.window_geom = WindowGeom {
                        is_topmost: false,
                        inner_size: Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() },
                        dpi_factor: zerde_parser.parse_f32(),
                        outer_size: Vec2 { x: 0., y: 0. },
                        position: Vec2 { x: 0., y: 0. },
                        xr_is_presenting: zerde_parser.parse_u32() > 0,
                        xr_can_present: zerde_parser.parse_u32() > 0,
                        is_fullscreen: zerde_parser.parse_u32() > 0,
                        can_fullscreen: old_geom.can_fullscreen,
                    };
                    assert!(self.platform.window_geom.dpi_factor > 0.0);
                    let new_geom = self.platform.window_geom.clone();

                    if self.windows.len() > 0 {
                        self.windows[0].window_geom = self.platform.window_geom.clone();
                    }
                    if old_geom != new_geom {
                        self.wasm_event_handler(Event::WindowGeomChange(WindowGeomChangeEvent {
                            window_id: 0,
                            old_geom,
                            new_geom,
                        }));
                    }

                    // do our initial redraw and repaint
                    self.request_draw();
                }
                5 => {
                    // animation_frame
                    is_animation_frame = true;
                    if self.requested_next_frame {
                        self.call_next_frame_event();
                    }
                }
                6 => {
                    // finger_down
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let button = zerde_parser.parse_u32() as usize;
                    let digit = zerde_parser.parse_u32() as usize;
                    self.platform.fingers_down[digit] = true;
                    let is_touch = zerde_parser.parse_u32() > 0;
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::FingerDown(FingerDownEvent {
                        window_id: 0,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        handled: false,
                        digit,
                        button: get_mouse_button(button),
                        input_type: if is_touch { FingerInputType::Touch } else { FingerInputType::Mouse },
                        modifiers,
                        time,
                        tap_count: 0,
                    }));
                }
                7 => {
                    // finger_up
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let button = zerde_parser.parse_u32() as usize;
                    let digit = zerde_parser.parse_u32() as usize;
                    self.platform.fingers_down[digit] = false;
                    let is_touch = zerde_parser.parse_u32() > 0;
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::FingerUp(FingerUpEvent {
                        window_id: 0,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        abs_start: Vec2::default(),
                        rel_start: Vec2::default(),
                        digit,
                        button: get_mouse_button(button),
                        is_over: false,
                        input_type: if is_touch { FingerInputType::Touch } else { FingerInputType::Mouse },
                        modifiers,
                        time,
                    }));
                }
                8 => {
                    // finger_move
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let digit = zerde_parser.parse_u32() as usize;
                    let is_touch = zerde_parser.parse_u32() > 0;
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::FingerMove(FingerMoveEvent {
                        window_id: 0,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        abs_start: Vec2::default(),
                        rel_start: Vec2::default(),
                        is_over: false,
                        digit,
                        input_type: if is_touch { FingerInputType::Touch } else { FingerInputType::Mouse },
                        modifiers,
                        time,
                    }));
                }
                9 => {
                    // finger_hover
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::FingerHover(FingerHoverEvent {
                        any_down: false,
                        digit: 0,
                        window_id: 0,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        handled: false,
                        hover_state: HoverState::Over,
                        modifiers,
                        time,
                    }));
                }
                10 => {
                    // finger_scroll
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let scroll = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let is_wheel = zerde_parser.parse_u32() != 0;
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::FingerScroll(FingerScrollEvent {
                        window_id: 0,
                        digit: 0,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        handled_x: false,
                        handled_y: false,
                        scroll,
                        input_type: if is_wheel { FingerInputType::Mouse } else { FingerInputType::Touch },
                        modifiers,
                        time,
                    }));
                }
                11 => {
                    // finger_out
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::FingerHover(FingerHoverEvent {
                        window_id: 0,
                        digit: 0,
                        any_down: false,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        handled: false,
                        hover_state: HoverState::Out,
                        modifiers,
                        time,
                    }));
                }
                12 | 13 | 14 | 17 => {
                    // key_down | key_up | text_input | text_copy
                    let event = parse_keyboard_event_from_js(msg_type, &mut zerde_parser);
                    self.wasm_event_handler(event);
                }
                18 => {
                    // timer_fired
                    let timer_id = zerde_parser.parse_f64() as u64;
                    self.wasm_event_handler(Event::Timer(TimerEvent { timer_id }));
                }
                19 => {
                    // window_focus
                    let focus = zerde_parser.parse_u32();
                    if focus == 0 {
                        self.wasm_event_handler(Event::AppFocusLost);
                    } else {
                        self.wasm_event_handler(Event::AppFocus);
                    }
                }
                20 => {
                    // xr_update, TODO(JP): bring this back some day?
                    // let inputs_len = zerde_parser.parse_u32();
                    // let time = zerde_parser.parse_f64();
                    // let head_transform = zerde_parser.parse_transform();
                    // let mut left_input = XRInput::default();
                    // let mut right_input = XRInput::default();
                    // let mut other_inputs = Vec::new();
                    // for _ in 0..inputs_len {
                    //     let skip = zerde_parser.parse_u32();
                    //     if skip == 0 {
                    //         continue;
                    //     }
                    //     let mut input = XRInput::default();
                    //     input.active = true;
                    //     input.grip = zerde_parser.parse_transform();
                    //     input.ray = zerde_parser.parse_transform();

                    //     let hand = zerde_parser.parse_u32();
                    //     let num_buttons = zerde_parser.parse_u32() as usize;
                    //     input.num_buttons = num_buttons;
                    //     for i in 0..num_buttons {
                    //         input.buttons[i].pressed = zerde_parser.parse_u32() > 0;
                    //         input.buttons[i].value = zerde_parser.parse_f32();
                    //     }

                    //     let num_axes = zerde_parser.parse_u32() as usize;
                    //     input.num_axes = num_axes;
                    //     for i in 0..num_axes {
                    //         input.axes[i] = zerde_parser.parse_f32();
                    //     }

                    //     if hand == 1 {
                    //         left_input = input;
                    //     } else if hand == 2 {
                    //         right_input = input;
                    //     } else {
                    //         other_inputs.push(input);
                    //     }
                    // }
                    // // call the VRUpdate event
                    // self.wasm_event_handler(&mut Event::XRUpdate(XRUpdateEvent {
                    //     time,
                    //     head_transform,
                    //     last_left_input: self.platform.xr_last_left_input.clone(),
                    //     last_right_input: self.platform.xr_last_right_input.clone(),
                    //     left_input: left_input.clone(),
                    //     right_input: right_input.clone(),
                    //     other_inputs,
                    // }));

                    // self.platform.xr_last_left_input = left_input;
                    // self.platform.xr_last_right_input = right_input;
                }
                21 => {
                    // paint_dirty, only set the passes of the main window to dirty
                    self.passes[self.windows[0].main_pass_id.unwrap()].paint_dirty = true;
                }
                22 => {
                    //http_send_response
                    let signal_id = zerde_parser.parse_u32();
                    let success = zerde_parser.parse_u32();
                    let mut new_set = BTreeSet::new();
                    new_set.insert(match success {
                        1 => Cx::STATUS_HTTP_SEND_OK,
                        _ => Cx::STATUS_HTTP_SEND_FAIL,
                    });
                    self.signals.insert(Signal { signal_id: signal_id as usize }, new_set);
                }
                23 => {
                    // websocket_message
                    let data = zerde_parser.parse_vec_ptr();
                    let url = zerde_parser.parse_string();
                    self.wasm_event_handler(Event::WebSocketMessage(WebSocketMessageEvent { url, result: Ok(data) }));
                }
                24 => {
                    // websocket_error
                    let url = zerde_parser.parse_string();
                    let err = zerde_parser.parse_string();
                    self.wasm_event_handler(Event::WebSocketMessage(WebSocketMessageEvent { url, result: Err(err) }));
                }
                25 => {
                    // app_open_files
                    let len = zerde_parser.parse_u32();
                    let user_files: Vec<UserFile> = (0..len)
                        .map(|_| {
                            let id = zerde_parser.parse_u32() as usize;
                            let size = zerde_parser.parse_u64();
                            let basename = zerde_parser.parse_string();

                            UserFile { basename, file: UniversalFile::from_wasm_file(id, size) }
                        })
                        .collect();
                    self.wasm_event_handler(Event::AppOpenFiles(AppOpenFilesEvent { user_files }));
                }
                26 => {
                    // send_event_from_any_thread
                    let event_ptr = zerde_parser.parse_u64();
                    let event_box = unsafe { Box::from_raw(event_ptr as *mut Event) };
                    self.wasm_event_handler(*event_box);
                }
                27 => {
                    // dragenter
                    self.wasm_event_handler(Event::FileDragBegin);
                }
                28 => {
                    // dragleave
                    self.wasm_event_handler(Event::FileDragCancel);
                }
                29 => {
                    // dragover
                    let x = zerde_parser.parse_u32() as f32;
                    let y = zerde_parser.parse_u32() as f32;

                    self.wasm_event_handler(Event::FileDragUpdate(FileDragUpdateEvent { abs: Vec2 { x, y } }));
                }
                30 => {
                    // call_rust
                    let name = zerde_parser.parse_string();
                    let params = zerde_parser.parse_wrf_params();
                    let callback_id = zerde_parser.parse_u32();
                    self.wasm_event_handler(Event::SystemEvent(SystemEvent::WebRustCall(Some(WebRustCallEvent {
                        name,
                        params,
                        callback_id,
                    }))));
                }
                _ => {
                    panic!("Message unknown {}", msg_type);
                }
            };
        }

        self.call_signals();

        if is_animation_frame && self.requested_draw {
            self.call_draw_event();
        }
        self.call_signals();

        for window in &mut self.windows {
            window.window_state = match &window.window_state {
                CxWindowState::Create { title, add_drop_target_for_app_open_files, .. } => {
                    self.platform.zerde_eventloop_msgs.set_document_title(&title);
                    window.window_geom = self.platform.window_geom.clone();

                    if *add_drop_target_for_app_open_files {
                        self.platform.zerde_eventloop_msgs.enable_global_file_drop_target();
                    }

                    CxWindowState::Created
                }
                CxWindowState::Close => CxWindowState::Closed,
                CxWindowState::Created => CxWindowState::Created,
                CxWindowState::Closed => CxWindowState::Closed,
            };

            window.window_command = match &window.window_command {
                CxWindowCmd::XrStartPresenting => {
                    self.platform.zerde_eventloop_msgs.xr_start_presenting();
                    CxWindowCmd::None
                }
                CxWindowCmd::XrStopPresenting => {
                    self.platform.zerde_eventloop_msgs.xr_stop_presenting();
                    CxWindowCmd::None
                }
                CxWindowCmd::FullScreen => {
                    self.platform.zerde_eventloop_msgs.fullscreen();
                    CxWindowCmd::None
                }
                CxWindowCmd::NormalScreen => {
                    self.platform.zerde_eventloop_msgs.normalscreen();
                    CxWindowCmd::None
                }
                _ => CxWindowCmd::None,
            };
        }

        // check if we need to send a cursor
        if !self.down_mouse_cursor.is_none() {
            self.platform.zerde_eventloop_msgs.set_mouse_cursor(self.down_mouse_cursor.as_ref().unwrap().clone())
        } else if !self.hover_mouse_cursor.is_none() {
            self.platform.zerde_eventloop_msgs.set_mouse_cursor(self.hover_mouse_cursor.as_ref().unwrap().clone())
        } else {
            self.platform.zerde_eventloop_msgs.set_mouse_cursor(MouseCursor::Default);
        }

        let mut passes_todo = Vec::new();
        let mut windows_need_repaint = 0;
        self.compute_passes_to_repaint(&mut passes_todo, &mut windows_need_repaint);

        if is_animation_frame && passes_todo.len() > 0 {
            let mut zerde_webgl = ZerdeWebGLMessages::new();
            self.webgl_compile_shaders(&mut zerde_webgl);
            for pass_id in &passes_todo {
                match self.passes[*pass_id].dep_of.clone() {
                    CxPassDepOf::Window(_) => {
                        // find the accompanying render window
                        // its a render window
                        windows_need_repaint -= 1;
                        let dpi_factor = self.platform.window_geom.dpi_factor;
                        self.draw_pass_to_canvas(*pass_id, dpi_factor, &mut zerde_webgl);
                    }
                    CxPassDepOf::Pass(parent_pass_id) => {
                        let dpi_factor = self.get_delegated_dpi_factor(parent_pass_id);
                        self.draw_pass_to_texture(*pass_id, dpi_factor, &mut zerde_webgl);
                    }
                    CxPassDepOf::None => {
                        self.draw_pass_to_texture(*pass_id, 1.0, &mut zerde_webgl);
                    }
                }
            }
            zerde_webgl.end();
            self.platform.zerde_eventloop_msgs.run_webgl(zerde_webgl.take_ptr());
        }

        // request animation frame if still need to redraw, or repaint
        // we use request animation frame for that.
        if passes_todo.len() != 0 || self.requested_draw || self.requested_next_frame {
            self.platform.zerde_eventloop_msgs.request_animation_frame();
        }

        // mark the end of the message
        self.platform.zerde_eventloop_msgs.end();

        // Return wasm pointer to caller and create a new ZerdeEventloopMsgs.
        std::mem::replace(&mut self.platform.zerde_eventloop_msgs, ZerdeEventloopMsgs::new()).take_ptr()
    }

    fn wasm_event_handler(&mut self, mut event: Event) {
        self.process_pre_event(&mut event);
        self.call_event_handler(&mut event);
        self.process_post_event(&mut event);
    }

    pub fn call_rust_in_same_thread_sync(&self, zerde_ptr: u64) -> u64 {
        if let Some(func) = self.platform.call_rust_in_same_thread_sync_fn.read().unwrap().clone() {
            let mut zerde_parser = ZerdeParser::from(zerde_ptr);
            let name = zerde_parser.parse_string();
            let params = zerde_parser.parse_wrf_params();
            let return_params = func(&name, params);
            let mut zerde_builder = ZerdeBuilder::new();
            zerde_builder.build_wrf_params(return_params);
            zerde_builder.take_ptr()
        } else {
            panic!("call_rust_in_same_thread_sync called but no call_rust_in_same_thread_sync_fn was registered");
        }
    }
}

impl CxDesktopVsWasmCommon for Cx {
    /// See [`CxDesktopVsWasmCommon::get_default_window_size`] for documentation.
    fn get_default_window_size(&self) -> Vec2 {
        return self.platform.window_geom.inner_size;
    }

    /// See [`CxDesktopVsWasmCommon::file_write`] for documentation.
    fn file_write(&mut self, _path: &str, _data: &[u8]) {
        unimplemented!();
    }

    /// See [`CxDesktopVsWasmCommon::http_send`] for documentation.
    fn http_send(
        &mut self,
        verb: &str,
        path: &str,
        proto: &str,
        domain: &str,
        port: u16,
        content_type: &str,
        body: &[u8],
        signal: Signal,
    ) {
        self.platform.zerde_eventloop_msgs.http_send(verb, path, proto, domain, port, content_type, body, signal);
    }

    /// See [`CxDesktopVsWasmCommon::websocket_send`] for documentation.
    fn websocket_send(&mut self, url: &str, data: &[u8]) {
        self.platform.zerde_eventloop_msgs.websocket_send(url, data);
    }

    /// See [`CxDesktopVsWasmCommon::call_js`] for documentation.
    fn call_js(&mut self, name: &str, params: Vec<WrfParam>) {
        self.platform.zerde_eventloop_msgs.call_js(name, params);
    }

    /// See [`CxDesktopVsWasmCommon::return_to_js`] for documentation.
    fn return_to_js(&mut self, callback_id: u32, mut params: Vec<WrfParam>) {
        params.insert(0, format!("{}", callback_id).into_param());
        self.call_js("_wrflibReturnParams", params);
    }

    /// See [`CxDesktopVsWasmCommon::register_call_rust_in_same_thread_sync_fn`] for documentation.
    fn register_call_rust_in_same_thread_sync_fn(&mut self, func: CallRustInSameThreadSyncFn) {
        *self.platform.call_rust_in_same_thread_sync_fn.write().unwrap() = Some(func);
    }
}

impl CxPlatformCommon for Cx {
    /// See [`CxPlatformCommon::post_signal`] for documentation.
    fn post_signal(signal: Signal, status: StatusId) {
        // TODO(JP): Signals are overcomplicated; let's simplify them..
        if signal.signal_id != 0 {
            let mut signals = HashMap::new();
            let mut new_set = BTreeSet::new();
            new_set.insert(status);
            signals.insert(signal, new_set);
            Cx::send_event_from_any_thread(Event::Signal(SignalEvent { signals }));
        }
    }

    /// See [`CxPlatformCommon::show_text_ime`] for documentation.
    fn show_text_ime(&mut self, x: f32, y: f32) {
        self.platform.zerde_eventloop_msgs.show_text_ime(x, y);
    }

    /// See [`CxPlatformCommon::hide_text_ime`] for documentation.
    fn hide_text_ime(&mut self) {
        self.platform.zerde_eventloop_msgs.hide_text_ime();
    }

    /// See [`CxPlatformCommon::start_timer`] for documentation.
    fn start_timer(&mut self, interval: f64, repeats: bool) -> Timer {
        self.last_timer_id += 1;
        self.platform.zerde_eventloop_msgs.start_timer(self.last_timer_id, interval, repeats);
        Timer { timer_id: self.last_timer_id }
    }

    /// See [`CxPlatformCommon::stop_timer`] for documentation.
    fn stop_timer(&mut self, timer: &mut Timer) {
        if timer.timer_id != 0 {
            self.platform.zerde_eventloop_msgs.stop_timer(timer.timer_id);
            timer.timer_id = 0;
        }
    }

    /// See [`CxPlatformCommon::update_menu`] for documentation.
    fn update_menu(&mut self, _menu: &Menu) {}

    /// See [`CxPlatformCommon::update_menu`] for documentation.
    fn copy_text_to_clipboard(&mut self, text: &str) {
        self.platform.zerde_eventloop_msgs.text_copy_response(text);
    }

    fn send_event_from_any_thread(event: Event) {
        let event_ptr = Box::into_raw(Box::new(event));
        unsafe {
            _sendEventFromAnyThread(event_ptr as u64);
        }
    }
}

/// See https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button#return_value
fn get_mouse_button(button: usize) -> MouseButton {
    return match button {
        0 => MouseButton::Left,
        2 => MouseButton::Right,
        _ => MouseButton::Other,
    };
}

// storage buffers for graphics API related platform
pub(crate) struct CxPlatform {
    pub(crate) is_initialized: bool,
    pub(crate) window_geom: WindowGeom,
    pub(crate) zerde_eventloop_msgs: ZerdeEventloopMsgs,
    pub(crate) vertex_buffers: usize,
    pub(crate) index_buffers: usize,
    pub(crate) vaos: usize,
    pub(crate) fingers_down: Vec<bool>,
    call_rust_in_same_thread_sync_fn: RwLock<Option<CallRustInSameThreadSyncFn>>,
    // pub(crate) xr_last_left_input: XRInput,
    // pub(crate) xr_last_right_input: XRInput,
}

impl Default for CxPlatform {
    fn default() -> CxPlatform {
        CxPlatform {
            is_initialized: false,
            window_geom: WindowGeom::default(),
            zerde_eventloop_msgs: ZerdeEventloopMsgs::new(),
            vertex_buffers: 0,
            index_buffers: 0,
            vaos: 0,
            fingers_down: Vec::new(),
            call_rust_in_same_thread_sync_fn: RwLock::new(None),
            // xr_last_left_input: XRInput::default(),
            // xr_last_right_input: XRInput::default(),
        }
    }
}

impl CxPlatform {}

pub(crate) struct ZerdeEventloopMsgs {
    builder: ZerdeBuilder,
}

impl ZerdeEventloopMsgs {
    pub(crate) fn new() -> Self {
        Self { builder: ZerdeBuilder::new() }
    }

    fn take_ptr(self /* move! */) -> u64 {
        self.builder.take_ptr()
    }

    pub(crate) fn end(&mut self) {
        self.builder.send_u32(0);
    }

    pub(crate) fn run_webgl(&mut self, zerde_webgl_ptr: u64) {
        self.builder.send_u32(1);
        self.builder.send_u64(zerde_webgl_ptr);
    }

    pub(crate) fn log(&mut self, msg: &str) {
        self.builder.send_u32(2);
        self.builder.send_string(msg);
    }

    pub(crate) fn load_deps(&mut self, deps: Vec<String>) {
        self.builder.send_u32(3);
        self.builder.send_u32(deps.len() as u32);
        for dep in deps {
            self.builder.send_string(&dep);
        }
    }

    pub(crate) fn request_animation_frame(&mut self) {
        self.builder.send_u32(4);
    }

    pub(crate) fn set_document_title(&mut self, title: &str) {
        self.builder.send_u32(5);
        self.builder.send_string(title);
    }

    pub(crate) fn set_mouse_cursor(&mut self, mouse_cursor: MouseCursor) {
        self.builder.send_u32(6);
        self.builder.send_u32(mouse_cursor as u32);
    }

    pub(crate) fn show_text_ime(&mut self, x: f32, y: f32) {
        self.builder.send_u32(7);
        self.builder.send_f32(x);
        self.builder.send_f32(y);
    }

    pub(crate) fn hide_text_ime(&mut self) {
        self.builder.send_u32(8);
    }

    pub(crate) fn text_copy_response(&mut self, response: &str) {
        self.builder.send_u32(9);
        self.builder.send_string(response);
    }

    pub(crate) fn start_timer(&mut self, id: u64, interval: f64, repeats: bool) {
        self.builder.send_u32(10);
        self.builder.send_u32(if repeats { 1 } else { 0 });
        self.builder.send_f64(id as f64);
        self.builder.send_f64(interval);
    }

    pub(crate) fn stop_timer(&mut self, id: u64) {
        self.builder.send_u32(11);
        self.builder.send_f64(id as f64);
    }

    pub(crate) fn xr_start_presenting(&mut self) {
        self.builder.send_u32(12);
    }

    pub(crate) fn xr_stop_presenting(&mut self) {
        self.builder.send_u32(13);
    }

    pub(crate) fn http_send(
        &mut self,
        verb: &str,
        path: &str,
        proto: &str,
        domain: &str,
        port: u16,
        content_type: &str,
        body: &[u8],
        signal: Signal,
    ) {
        self.builder.send_u32(14);
        self.builder.send_u32(port as u32);
        self.builder.send_u32(signal.signal_id as u32);
        self.builder.send_string(verb);
        self.builder.send_string(path);
        self.builder.send_string(proto);
        self.builder.send_string(domain);
        self.builder.send_string(content_type);
        self.builder.send_u8slice(body);
    }

    pub(crate) fn fullscreen(&mut self) {
        self.builder.send_u32(15);
    }

    pub(crate) fn normalscreen(&mut self) {
        self.builder.send_u32(16);
    }

    pub(crate) fn websocket_send(&mut self, url: &str, data: &[u8]) {
        self.builder.send_u32(17);
        self.builder.send_string(url);
        self.builder.send_u8slice(data);
    }

    pub(crate) fn enable_global_file_drop_target(&mut self) {
        self.builder.send_u32(18);
    }

    pub(crate) fn call_js(&mut self, name: &str, params: Vec<WrfParam>) {
        self.builder.send_u32(19);
        self.builder.send_string(name);

        self.builder.build_wrf_params(params);
    }
}

// for use with sending wasm vec data
#[export_name = "allocWasmVec"]
pub unsafe extern "C" fn alloc_wasm_vec(bytes: u64) -> u64 {
    let mut vec = vec![0u8; bytes as usize];
    // let mut vec = Vec::<u8>::with_capacity(bytes as usize);
    // vec.resize(bytes as usize, 0);
    let ptr = vec.as_mut_ptr();
    mem::forget(vec);
    return ptr as u64;
}

// for use with message passing
#[export_name = "allocWasmMessage"]
pub unsafe extern "C" fn alloc_wasm_message(bytes: u64) -> u64 {
    let buf = std::alloc::alloc(std::alloc::Layout::from_size_align(bytes as usize, mem::align_of::<u64>()).unwrap()) as usize;
    (buf as *mut u64).write(bytes as u64);
    buf as u64
}

// for use with message passing
#[export_name = "reallocWasmMessage"]
pub unsafe extern "C" fn realloc_wasm_message(in_buf: u64, new_bytes: u64) -> u64 {
    let old_buf = in_buf as *mut u8;
    let old_bytes = (old_buf as *mut u64).read() as usize;
    let new_buf = alloc::alloc(alloc::Layout::from_size_align(new_bytes as usize, mem::align_of::<u64>()).unwrap()) as *mut u8;
    ptr::copy_nonoverlapping(old_buf, new_buf, old_bytes);
    alloc::dealloc(old_buf as *mut u8, alloc::Layout::from_size_align(old_bytes as usize, mem::align_of::<u64>()).unwrap());
    (new_buf as *mut u64).write(new_bytes as u64);
    new_buf as u64
}

#[export_name = "deallocWasmMessage"]
pub unsafe extern "C" fn dealloc_wasm_message(in_buf: u64) {
    let buf = in_buf as *mut u8;
    let bytes = buf.read() as usize;
    std::alloc::dealloc(buf as *mut u8, std::alloc::Layout::from_size_align(bytes as usize, mem::align_of::<u64>()).unwrap());
}

fn create_arc_vec_inner<T>(vec_ptr: u64, vec_len: u64) -> u64 {
    let vec: Vec<T> = unsafe { Vec::from_raw_parts(vec_ptr as *mut T, vec_len as usize, vec_len as usize) };
    let arc = Arc::new(vec);
    Arc::into_raw(arc) as u64
}

#[export_name = "createArcVec"]
pub unsafe extern "C" fn create_arc_vec(vec_ptr: u64, vec_len: u64, param_type: u64) -> u64 {
    match param_type as u32 {
        WRF_PARAM_READ_ONLY_UINT8_BUFFER => create_arc_vec_inner::<u8>(vec_ptr, vec_len),
        WRF_PARAM_READ_ONLY_FLOAT32_BUFFER => create_arc_vec_inner::<f32>(vec_ptr, vec_len),
        v => panic!("create_arc_vec: Invalid param type: {}", v),
    }
}

#[export_name = "incrementArc"]
pub unsafe extern "C" fn increment_arc(arc_ptr: u64) {
    Arc::increment_strong_count(arc_ptr as usize as *const Vec<u8>);
}

#[export_name = "decrementArc"]
pub unsafe extern "C" fn decrement_arc(arc_ptr: u64) {
    Arc::decrement_strong_count(arc_ptr as usize as *const Vec<u8>);
}

#[export_name = "deallocVec"]
pub unsafe extern "C" fn dealloc_vec(vec_ptr: u64, vec_len: u64, vec_cap: u64) {
    let vec: Vec<u8> = Vec::from_raw_parts(vec_ptr as *mut u8, vec_len as usize, vec_cap as usize);
    drop(vec);
}

extern "C" {
    fn _consoleLog(chars: u64, len: u64, error: bool);
    pub fn performanceNow() -> f64;
    fn _sendEventFromAnyThread(event_ptr: u64);
}

pub fn console_log(val: &str) {
    unsafe {
        let chars = val.chars().collect::<Vec<char>>();
        _consoleLog(chars.as_ptr() as u64, chars.len() as u64, false);
    }
}

pub fn console_error(val: &str) {
    unsafe {
        let chars = val.chars().collect::<Vec<char>>();
        _consoleLog(chars.as_ptr() as u64, chars.len() as u64, true);
    }
}

extern "C" {
    fn sendTaskWorkerMessage(tw_message_ptr: u64);
}

pub(crate) const TASK_WORKER_INITIAL_RETURN_VALUE: i32 = -1;
pub(crate) const TASK_WORKER_ERROR_RETURN_VALUE: i32 = -2;

/// Opens a new HTTP stream, blocks until there's a successful response, and returns a stream id.
pub(crate) fn send_task_worker_message_http_stream_new(url: &str, method: &str, body: &[u8], headers: &[(&str, &str)]) -> i32 {
    let mut stream_id = TASK_WORKER_INITIAL_RETURN_VALUE;
    let mut zerde_builder = ZerdeBuilder::new();
    zerde_builder.send_u32(1); // Message type.
    zerde_builder.send_u32(&mut stream_id as *mut i32 as u32);
    zerde_builder.send_string(url);
    zerde_builder.send_string(method);
    zerde_builder.send_u8slice(body);
    zerde_builder.send_u32(headers.len() as u32);
    for (name, value) in headers {
        zerde_builder.send_string(name);
        zerde_builder.send_string(value);
    }
    let zerde_ptr = zerde_builder.take_ptr();
    unsafe {
        sendTaskWorkerMessage(zerde_ptr);
        // Wait until the task worker sets `stream_id` to a return value.
        core::arch::wasm32::memory_atomic_wait32(&mut stream_id as *mut i32, TASK_WORKER_INITIAL_RETURN_VALUE, -1);
        dealloc_wasm_message(zerde_ptr);
    }
    stream_id
}

/// Makes a read call for an HTTP stream, blocking until its fulfilled, and returning the number of bytes read
/// (or 0 when we're at the end of the stream).
pub(crate) fn send_task_worker_message_http_stream_read(stream_id: i32, buf_ptr: *mut u8, buf_len: usize) -> i32 {
    let mut bytes_read = TASK_WORKER_INITIAL_RETURN_VALUE;
    let mut zerde_builder = ZerdeBuilder::new();
    zerde_builder.send_u32(2); // Message type.
    zerde_builder.send_u32(&mut bytes_read as *mut i32 as u32);
    zerde_builder.send_u32(stream_id as u32);
    zerde_builder.send_u32(buf_ptr as u32);
    zerde_builder.send_u32(buf_len as u32);
    let zerde_ptr = zerde_builder.take_ptr();
    unsafe {
        sendTaskWorkerMessage(zerde_ptr);
        // Wait until the task worker sets `bytes_read` to a return value.
        core::arch::wasm32::memory_atomic_wait32(&mut bytes_read as *mut i32, TASK_WORKER_INITIAL_RETURN_VALUE, -1);
        dealloc_wasm_message(zerde_ptr);
    }
    bytes_read
}
