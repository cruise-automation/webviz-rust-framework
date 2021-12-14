// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::background::*;
use crate::buttonlogic::*;
use crate::desktopbutton::*;
use wrflib::*;

pub struct DesktopWindow {
    pub window: Window,
    pub pass: Pass,
    pub clear_color: Vec4,
    pub color_texture: Texture,
    pub depth_texture: Texture,
    pub caption_view: View, // we have a root view otherwise is_overlay subviews can't attach topmost
    pub main_view: View,    // we have a root view otherwise is_overlay subviews can't attach topmost
    pub inner_view: View,
    pub inner_layout: Layout,
    //pub caption_bg_color: ColorId,
    pub min_btn: DesktopButton,
    pub max_btn: DesktopButton,
    pub close_btn: DesktopButton,
    pub xr_btn: DesktopButton,
    pub fullscreen_btn: DesktopButton,
    pub caption_bg: Background,
    pub caption_size: Vec2,
    pub caption: String,

    pub default_menu: Menu,

    // testing
    pub inner_over_chrome: bool,
}

#[derive(Clone, PartialEq)]
pub enum DesktopWindowEvent {
    EventForOtherWindow,
    WindowClosed,
    WindowGeomChange(WindowGeomChangeEvent),
    None,
}

impl DesktopWindow {
    pub fn new() -> Self {
        Self {
            window: Window::default(),
            pass: Pass::default(),
            clear_color: Vec4::color("1e"),
            color_texture: Texture::default(),
            depth_texture: Texture::default(),
            main_view: View::default(),
            caption_view: View::default(),
            inner_view: View::default(),
            inner_layout: Layout::default(),
            min_btn: DesktopButton::default(),
            max_btn: DesktopButton::default(),
            close_btn: DesktopButton::default(),
            xr_btn: DesktopButton::default(),
            fullscreen_btn: DesktopButton::default(),

            default_menu: Menu::main(vec![Menu::sub("App", vec![Menu::item("Quit App", Cx::COMMAND_QUIT)])]),
            //caption_bg_color: Color_bg_selected_over::id(cx),
            caption_bg: Background::default(),
            caption_size: Vec2::default(),
            caption: "Bigedit".to_string(),
            inner_over_chrome: false,
        }
    }

    pub fn with_window(self, window: Window) -> Self {
        Self { window, ..self }
    }
    pub fn with_caption(self, caption: &str) -> Self {
        Self { caption: caption.to_string(), ..self }
    }

    pub fn with_inner_layout(self, inner_layout: Layout) -> Self {
        Self { inner_layout, ..self }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> DesktopWindowEvent {
        //self.main_view.handle_scroll_bars(cx, event);
        //self.inner_view.handle_scroll_bars(cx, event);
        if let ButtonEvent::Clicked = self.xr_btn.handle(cx, event) {
            if self.window.xr_is_presenting(cx) {
                self.window.xr_stop_presenting(cx);
            } else {
                self.window.xr_start_presenting(cx);
            }
        }

        if let ButtonEvent::Clicked = self.fullscreen_btn.handle(cx, event) {
            if self.window.is_fullscreen(cx) {
                self.window.normal_window(cx);
            } else {
                self.window.fullscreen_window(cx);
            }
        }
        if let ButtonEvent::Clicked = self.min_btn.handle(cx, event) {
            self.window.minimize_window(cx);
        }
        if let ButtonEvent::Clicked = self.max_btn.handle(cx, event) {
            if self.window.is_fullscreen(cx) {
                self.window.restore_window(cx);
            } else {
                self.window.maximize_window(cx);
            }
        }
        if let ButtonEvent::Clicked = self.close_btn.handle(cx, event) {
            self.window.close_window(cx);
        }
        if let Some(window_id) = self.window.window_id {
            let is_for_other_window = match event {
                Event::WindowCloseRequested(ev) => ev.window_id != window_id,
                Event::WindowClosed(ev) => {
                    if ev.window_id == window_id {
                        return DesktopWindowEvent::WindowClosed;
                    }
                    true
                }
                Event::WindowGeomChange(ev) => {
                    if ev.window_id == window_id {
                        return DesktopWindowEvent::WindowGeomChange(ev.clone());
                    }
                    true
                }
                Event::WindowDragQuery(dq) => {
                    if dq.window_id == window_id && dq.abs.x < self.caption_size.x && dq.abs.y < self.caption_size.y {
                        if dq.abs.x < 50. {
                            dq.response = WindowDragQueryResponse::SysMenu;
                        } else {
                            dq.response = WindowDragQueryResponse::Caption;
                        }
                    }
                    true
                }
                Event::FingerDown(ev) => ev.window_id != window_id,
                Event::FingerMove(ev) => ev.window_id != window_id,
                Event::FingerHover(ev) => ev.window_id != window_id,
                Event::FingerUp(ev) => ev.window_id != window_id,
                Event::FingerScroll(ev) => ev.window_id != window_id,
                _ => false,
            };
            if is_for_other_window {
                DesktopWindowEvent::EventForOtherWindow
            } else {
                DesktopWindowEvent::None
            }
        } else {
            DesktopWindowEvent::None
        }
    }

    pub fn begin_draw(&mut self, cx: &mut Cx, menu: Option<&Menu>) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, self.clear_color);

        let _ = self.main_view.begin_view(cx, Layout::default());

        self.caption_view.begin_view(cx, Layout { walk: Walk::wh(Width::Fill, Height::Compute), ..Layout::default() });

        let process_chrome = match cx.platform_type {
            PlatformType::Linux { custom_window_chrome } => custom_window_chrome,
            _ => true,
        };
        if process_chrome {
            let color = vec4(0.24, 0.24, 0.24, 1.0);

            match cx.platform_type {
                PlatformType::Windows | PlatformType::Unknown | PlatformType::Linux { .. } => {
                    let caption_turtle = self.caption_bg.begin_turtle(
                        cx,
                        Layout { walk: Walk::wh(Width::Fill, Height::Compute), ..Default::default() },
                        color,
                    );

                    // we need to draw the window menu here.
                    if let Some(_menu) = menu {
                        // lets draw the thing, check with the clone if it changed
                        // then draw it
                    }
                    cx.begin_right_align();
                    self.min_btn.draw(cx, DesktopButtonType::WindowsMin);
                    if self.window.is_fullscreen(cx) {
                        self.max_btn.draw(cx, DesktopButtonType::WindowsMaxToggled);
                    } else {
                        self.max_btn.draw(cx, DesktopButtonType::WindowsMax);
                    }
                    self.close_btn.draw(cx, DesktopButtonType::WindowsClose);
                    cx.end_right_align();

                    cx.begin_center_x_and_y_align();
                    self.caption_size = Vec2 { x: cx.get_width_left(), y: cx.get_height_left() };
                    TextIns::draw_walk(cx, &self.caption, &TextInsProps::DEFAULT);
                    cx.end_center_x_and_y_align();

                    // we need to store our caption rect somewhere.
                    self.caption_bg.end_turtle(cx, caption_turtle);
                    cx.turtle_new_line();
                }

                PlatformType::OSX => {
                    // mac still uses the built in buttons, TODO, replace that.
                    if let Some(menu) = menu {
                        cx.update_menu(menu);
                    } else {
                        cx.update_menu(&self.default_menu);
                    }
                    let caption_turtle = self.caption_bg.begin_turtle(
                        cx,
                        Layout { walk: Walk::wh(Width::Fill, Height::Fix(22.)), ..Default::default() },
                        color,
                    );
                    cx.begin_center_x_and_y_align();
                    self.caption_size = Vec2 { x: cx.get_width_left(), y: cx.get_height_left() };
                    TextIns::draw_walk(cx, &self.caption, &TextInsProps::DEFAULT);
                    cx.end_center_x_and_y_align();
                    self.caption_bg.end_turtle(cx, caption_turtle);
                    cx.turtle_new_line();
                }
                PlatformType::Web { .. } => {
                    if self.window.is_fullscreen(cx) {
                        // put a bar at the top
                        let rect = cx.walk_turtle(Walk::wh(Width::Fill, Height::Fix(22.)));
                        self.caption_bg.draw(cx, rect, color);
                        cx.turtle_new_line();
                    }
                }
            }
        }
        self.caption_view.end_view(cx);
        cx.turtle_new_line();

        if self.inner_over_chrome {
            self.inner_view.begin_view(cx, Layout { absolute: true, ..self.inner_layout });
        } else {
            self.inner_view.begin_view(cx, self.inner_layout);
        }
    }

    pub fn end_draw(&mut self, cx: &mut Cx) {
        self.inner_view.end_view(cx);
        // lets draw a VR button top right over the UI.
        // window fullscreen?

        // only support fullscreen on web atm
        if !cx.platform_type.is_desktop() && !self.window.is_fullscreen(cx) {
            cx.reset_turtle_pos();
            cx.move_turtle(cx.get_width_total() - 50.0, 0.);
            self.fullscreen_btn.draw(cx, DesktopButtonType::Fullscreen);
        }

        if self.window.xr_can_present(cx) {
            // show a switch-to-VRMode button
            cx.reset_turtle_pos();
            cx.move_turtle(cx.get_width_total() - 100.0, 0.);
            self.xr_btn.draw(cx, DesktopButtonType::XRMode);
        }

        self.main_view.end_view(cx);

        self.pass.end_pass(cx);

        self.window.end_window(cx);
    }
}
