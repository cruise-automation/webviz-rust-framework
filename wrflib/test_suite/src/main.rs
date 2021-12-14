// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use std::sync::Arc;

use wrflib::*;
use wrflib_widget::*;

pub struct TestSuiteApp {
    window: Window,
    pass: Pass,
    main_view: View,
    send_button: NormalButton,
    worker_button: NormalButton,
    dump_button: NormalButton,
    buffers: Vec<Arc<Vec<u8>>>,
}

fn array_multiply(params: &[WrfParam]) -> Vec<WrfParam> {
    let value: u8 = serde_json::from_str(params[0].as_string()).unwrap();
    let mut outputs = Vec::new();
    for p in params[1..].iter() {
        let buffer = match p {
            WrfParam::String(_) => panic!(),
            WrfParam::ReadOnlyBuffer(b) => b,
            WrfParam::Buffer(b) => b,
        };
        let output: Vec<_> = buffer.iter().map(|x| *x * value).collect();
        outputs.push(WrfParam::ReadOnlyBuffer(Arc::new(output)));
    }

    outputs
}

impl TestSuiteApp {
    pub fn new(cx: &mut Cx) -> Self {
        cx.register_call_rust_in_same_thread_sync_fn(Self::call_rust_in_same_thread_sync);
        cx.on_call_rust(Self::on_call_rust);
        let buffer = Arc::new(vec![1; 8]);
        let buffers = vec![buffer];
        Self {
            window: Window {
                create_inner_size: Some(vec2(1000., 700.)),
                #[cfg(not(target_arch = "wasm32"))]
                create_cef_url: Some("http://localhost:3000/wrflib/test_suite/index.html".to_string()),
                ..Window::default()
            },
            pass: Pass::default(),
            main_view: View::default(),
            send_button: NormalButton::default(),
            worker_button: NormalButton::default(),
            dump_button: NormalButton::default(),
            buffers,
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        match event {
            Event::Signal(_) => {
                log!("received signal!");
            }
            _ => {}
        }

        if let ButtonEvent::Clicked = self.send_button.handle(cx, event) {
            let mut params = vec![WrfParam::String("hello world :-)".to_string())];
            for buffer in &self.buffers {
                params.push(WrfParam::ReadOnlyBuffer(buffer.clone()));
            }
            cx.call_js("log", params);
        }
        if let ButtonEvent::Clicked = self.worker_button.handle(cx, event) {
            let buffer = Arc::new(vec![1; 8]);
            let params = vec![WrfParam::ReadOnlyBuffer(Arc::clone(&buffer))];
            cx.call_js("sendWorker", params);
            self.buffers.push(buffer);
        }
        if let ButtonEvent::Clicked = self.dump_button.handle(cx, event) {
            log!("Total buffers: {}", self.buffers.len());
            for buffer in self.buffers.iter() {
                let count = Arc::strong_count(buffer);
                if count > 1 {
                    log!("Buffer arc={} ptr={}, rc = {}", Arc::as_ptr(buffer) as u32, buffer.as_ptr() as u32, count);
                }
            }
        }
    }

    fn on_call_rust(&mut self, _cx: &mut Cx, name: String, params: Vec<WrfParam>) -> Vec<WrfParam> {
        match name.as_str() {
            "array_multiply" => array_multiply(&params),
            "total_sum" => {
                let buffer = params[0].as_buffer();
                let sum: u8 = buffer.iter().sum();
                vec![WrfParam::String(sum.to_string())]
            }
            "call_rust_no_return" => {
                // Note: not returning anything to test destructor behavior
                vec![]
            }
            // TODO(JP): Turn this into an actual API.
            "make_wrfbuffer" => {
                vec![WrfParam::ReadOnlyBuffer(Arc::new(vec![1, 2, 3, 4, 5, 6, 7, 8]))]
            }
            "make_mutable_wrfbuffer" => {
                vec![WrfParam::Buffer(vec![1, 2, 3, 4, 5, 6, 7, 8])]
            }
            "check_arc_count" => {
                let arc_ptr = params[0].as_string().parse::<u64>().unwrap() as *const Vec<u8>;
                let arc: Arc<Vec<u8>> = unsafe { Arc::from_raw(arc_ptr) };
                let count = Arc::strong_count(&arc);
                Arc::into_raw(arc);
                vec![WrfParam::Buffer(vec![count as u8])]
            }
            unknown_name => {
                panic!("Unknown function name: {}", unknown_name)
            }
        }
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::all(0.));

        self.main_view.begin_view(cx, Layout::default());
        let main_turtle = cx.begin_turtle(Layout { direction: Direction::Down, ..Layout::default() });

        cx.walk_turtle(Walk::wh(Width::Fix(0.), Height::Fix(30.)));
        cx.begin_right_align();
        self.send_button.draw(cx, "send log event");
        self.worker_button.draw(cx, "send to worker");
        self.dump_button.draw(cx, "dump rc counts");
        cx.end_right_align();
        cx.end_turtle(main_turtle);
        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }

    pub fn call_rust_in_same_thread_sync(name: &str, params: Vec<WrfParam>) -> Vec<WrfParam> {
        match name {
            "array_multiply" => array_multiply(&params),
            "make_wrfbuffer" => {
                vec![WrfParam::ReadOnlyBuffer(Arc::new(vec![1, 2, 3, 4, 5, 6, 7, 8]))]
            }
            "send_signal" => {
                // This is a fake signal ID
                Cx::post_signal(Signal { signal_id: 123 }, location_hash!());
                Vec::new()
            }
            unknown => {
                panic!("Unknown function name: {}", unknown);
            }
        }
    }
}

main_app!(TestSuiteApp);