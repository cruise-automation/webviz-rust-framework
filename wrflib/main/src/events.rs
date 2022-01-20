// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

//! Events coming from user actions, system calls, and so on.

use crate::{universal_file::UniversalFile, *};
use std::collections::{BTreeSet, HashMap};

/// Modifiers that were held when a key event was fired.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub logo: bool,
}

/// The type of input that was used to trigger a finger event.
#[derive(Clone, Debug, PartialEq)]
pub enum FingerInputType {
    Mouse,
    Touch,
    XR,
}

impl FingerInputType {
    pub fn is_touch(&self) -> bool {
        *self == FingerInputType::Touch
    }
    pub fn is_mouse(&self) -> bool {
        *self == FingerInputType::Mouse
    }
    pub fn is_xr(&self) -> bool {
        *self == FingerInputType::XR
    }
    pub fn has_hovers(&self) -> bool {
        *self == FingerInputType::Mouse || *self == FingerInputType::XR
    }
}

impl Default for FingerInputType {
    fn default() -> Self {
        Self::Mouse
    }
}

/// The type of input that was used to trigger a finger event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Other,
}

impl Default for MouseButton {
    fn default() -> Self {
        Self::Left
    }
}

/// A conceptual "finger" (mouse, actual finger, etc) was pressed down.
///
/// Someone has to call [`Cx::set_key_focus`] or [`Cx::keep_key_focus`] when handling `FingerDown`, otherwise
/// the key focus will be reset.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FingerDownEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub rel: Vec2,
    pub rect: Rect,
    // Digit based system is supposed to track finger interactions, where each finger is a digit.
    // Its doesn't exactly work in the way its supposed to work in terms of finger tracking.
    // Our needs currently require us to have sure shot Mouse interaction events.
    // Hence we are adding `button: MouseButton` in addition to existing digits.
    // TODO(Shobhit): Refresh the digit based finger tracking system someday.
    pub digit: usize,
    pub button: MouseButton,
    pub tap_count: u32,
    pub(crate) handled: bool,
    pub input_type: FingerInputType,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

/// A conceptual "finger" (mouse, actual finger, etc) was moved.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FingerMoveEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub abs_start: Vec2,
    pub rel: Vec2,
    pub rel_start: Vec2,
    pub rect: Rect,
    pub is_over: bool,
    pub digit: usize,
    pub input_type: FingerInputType,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

impl FingerMoveEvent {
    pub fn move_distance(&self) -> f32 {
        ((self.abs_start.x - self.abs.x).powf(2.) + (self.abs_start.y - self.abs.y).powf(2.)).sqrt()
    }
}

/// A conceptual "finger" (mouse, actual finger, etc) was released.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FingerUpEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub abs_start: Vec2,
    pub rel: Vec2,
    pub rel_start: Vec2,
    pub rect: Rect,
    pub digit: usize,
    pub button: MouseButton,
    pub is_over: bool,
    pub input_type: FingerInputType,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

/// The type of [`FingerHoverEvent`].
#[derive(Clone, Debug, PartialEq)]
pub enum HoverState {
    In,
    Over,
    Out,
}

impl Default for HoverState {
    fn default() -> HoverState {
        HoverState::Over
    }
}

/// A conceptual "finger" (mouse, actual finger, etc) was hovered over the screen.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FingerHoverEvent {
    pub window_id: usize,
    pub digit: usize,
    pub abs: Vec2,
    pub rel: Vec2,
    pub rect: Rect,
    pub any_down: bool,
    pub(crate) handled: bool,
    pub hover_state: HoverState,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

/// A conceptual "finger" (mouse, actual finger, etc) triggered a scroll.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FingerScrollEvent {
    pub window_id: usize,
    pub digit: usize,
    pub abs: Vec2,
    pub rel: Vec2,
    pub rect: Rect,
    pub scroll: Vec2,
    pub input_type: FingerInputType,
    //pub is_wheel: bool,
    pub handled_x: bool,
    pub handled_y: bool,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

/// Geometry of a [`Window`] changed (position, size, etc).
#[derive(Clone, Default, Debug, PartialEq)]
pub struct WindowGeomChangeEvent {
    pub window_id: usize,
    pub old_geom: WindowGeom,
    pub new_geom: WindowGeom,
}

/// A [`Timer`] that was requested using [`Cx::start_timer`] has fired.
#[derive(Clone, Debug, PartialEq)]
pub struct TimerEvent {
    pub timer_id: u64,
}

/// Represents a signal that was fired from [`Cx::send_signal`]. Can be captured
/// with [`Signal`].
///
/// TODO(JP): Is this a bit too complicated of an API? What about if we just
/// send `pub signal: u64`, or even a `Box`? Then you can use it for anything.
#[derive(Clone, Debug, PartialEq)]
pub struct SignalEvent {
    pub signals: HashMap<Signal, BTreeSet<StatusId>>,
}

/// Data for various kinds of key-based events ([`Event::KeyDown`], [`Event::KeyUp`], etc).
#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    pub key_code: KeyCode,
    //pub key_char: char,
    pub is_repeat: bool,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

/// Called when [`Cx::key_focus`] changes.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyFocusEvent {
    pub(crate) prev: Option<ComponentId>,
    pub(crate) focus: Option<ComponentId>,
}

/// When a file is being dragged and the mouse position changes
#[derive(Clone, Debug, PartialEq)]
pub struct FileDragUpdateEvent {
    pub abs: Vec2,
}

/// Some text was inputted. Rely on this for text input instead of [`KeyEvent`]s.
#[derive(Clone, Debug, PartialEq)]
pub struct TextInputEvent {
    pub input: String,
    pub replace_last: bool,
    pub was_paste: bool,
}

/// The user requested to close the [`Window`].
#[derive(Clone, Debug, PartialEq)]
pub struct WindowCloseRequestedEvent {
    pub window_id: usize,
    pub accept_close: bool,
}

/// The [`Window`] actually closed.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowClosedEvent {
    pub window_id: usize,
}

/// The user started or ended resizing the [`Window`].
///
/// TODO(JP): Mostly for internal use in Windows; we might not want to expose this
/// to end users?
#[derive(Clone, Debug, PartialEq)]
pub struct WindowResizeLoopEvent {
    pub was_started: bool,
    pub window_id: usize,
}

/// Response to operating system inquiry if a [`Window`] can be dragged.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowDragQueryResponse {
    NoAnswer,
    Client,
    Caption,
    SysMenu, // windows only
}

/// The operating system inquired if a [`Window`] can be dragged.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowDragQueryEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub response: WindowDragQueryResponse,
}

/// A websocket message was received.
#[derive(Clone, Debug, PartialEq)]
pub struct WebSocketMessageEvent {
    pub url: String,
    pub result: Result<Vec<u8>, String>,
}

/// A file that was supplied by a user, as opposed to by the application itself (like font resources
/// and such).
#[derive(Clone, Debug)]
pub struct UserFile {
    /// Per UNIX convention, basename is the filename (including extension) part.
    /// This is the only part of the filename that is exposed on all platforms (Wasm hides the
    /// full path).
    pub basename: String,
    /// The actual file handle.
    pub file: UniversalFile,
}
impl PartialEq for UserFile {
    fn eq(&self, other: &Self) -> bool {
        self.basename == other.basename
    }
}

/// Fires when a web worker calls `callRust` to trigger a function in Rust.
#[derive(Clone, Debug)]
pub struct WebRustCallEvent {
    /// Description of the event.
    pub name: String,
    pub params: Vec<WrfParam>,
    pub callback_id: u32,
}

/// This is an application-level event intended for platforms that can register a file type to an application.
/// Fires:
/// - when application starts with a file
/// - when `Window::create_add_drop_target_for_app_open_files` is set and a file is dragged and released onto the window
/// - when the application is already started and an associated file is double-clicked
#[derive(Clone, Debug, PartialEq)]
pub struct AppOpenFilesEvent {
    pub user_files: Vec<UserFile>,
}

/// Events that are handled internally and are not propagated to an application `handle` method.
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// See [`WebRustCallEvent`]. This event must have a handler registered through [`Cx::on_call_rust`].
    WebRustCall(Option<WebRustCallEvent>),
    Draw,
    /// We're going to repaint our draw tree.
    Paint,
    /// The system wants us to set a different mouse cursor.
    WindowSetHoverCursor(MouseCursor),
    /// Calls `do_message_loop_work` for CEF.
    #[cfg(feature = "cef")]
    CefDoMessageLoopWork,
}

/// Global event that gets passed into `handle`, and which you can pass down to your own widgets.
#[derive(Clone, Debug)]
pub enum Event {
    /// No event, to avoid `[Option<Event>]` all over the place.
    None,
    /// App gets started. Should be the very first event that gets fired.
    Construct,
    /// App gained focus.
    ///
    /// TODO(JP): Rename to `AppFocusGained` to be more symmetric with [`Event::AppFocusLost`]?
    AppFocus,
    /// App lost focus.
    AppFocusLost,
    /// We're going to paint a new frame. Useful for animations; you can request this using [`Cx::request_next_frame`].
    NextFrame,
    /// See [`WindowDragQueryEvent`]
    WindowDragQuery(WindowDragQueryEvent),
    /// See [`WindowCloseRequestedEvent`]
    WindowCloseRequested(WindowCloseRequestedEvent),
    /// See [`WindowClosedEvent`]
    WindowClosed(WindowClosedEvent),
    /// See [`WindowGeomChangeEvent`]
    WindowGeomChange(WindowGeomChangeEvent),
    /// See [`WindowResizeLoopEvent`]
    WindowResizeLoop(WindowResizeLoopEvent),
    /// See [`FingerDownEvent`]
    FingerDown(FingerDownEvent),
    /// See [`FingerMoveEvent`]
    FingerMove(FingerMoveEvent),
    /// See [`FingerHoverEvent`]
    FingerHover(FingerHoverEvent),
    /// See [`FingerUpEvent`]
    FingerUp(FingerUpEvent),
    /// See [`FingerScrollEvent`]
    FingerScroll(FingerScrollEvent),
    /// See [`TimerEvent`]
    Timer(TimerEvent),
    /// See [`SignalEvent`]
    Signal(SignalEvent),
    /// See [`CommandId`]
    Command(CommandId),
    /// See [`KeyFocusEvent`]
    KeyFocus(KeyFocusEvent),
    /// See [`KeyFocusEvent`]
    KeyFocusLost(KeyFocusEvent),
    /// User pressed down a key. See also [`KeyEvent`]
    KeyDown(KeyEvent),
    /// User released a key. See also [`KeyEvent`]
    KeyUp(KeyEvent),
    /// See [`TextInputEvent`]
    TextInput(TextInputEvent),
    /// The user requested text to be copied to the clipboard.
    TextCopy,
    /// See [`WebSocketMessageEvent`]
    WebSocketMessage(WebSocketMessageEvent),
    /// See [`AppOpenFilesEvent`]
    AppOpenFiles(AppOpenFilesEvent),
    /// When `Window::create_add_drop_target_for_app_open_files` is set and a file is dragged (without being released)
    /// onto the window
    FileDragBegin,
    /// See [`FileDragUpdateEvent`]
    FileDragUpdate(FileDragUpdateEvent),
    /// When a file is being dragged and the mouse moves out of the window
    FileDragCancel,
    /// See [`SystemEvent`]. These events are not passed to `handle`.
    SystemEvent(SystemEvent),
}

impl Default for Event {
    fn default() -> Event {
        Event::None
    }
}

/// A margin around the area that is checked for event hits
#[derive(Clone, Copy, Debug)]
pub struct Margin {
    pub l: f32,
    pub t: f32,
    pub r: f32,
    pub b: f32,
}
impl Margin {
    pub const ZERO: Margin = Margin { l: 0.0, t: 0.0, r: 0.0, b: 0.0 };

    /// TODO(JP): Replace these with Margin::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Margin = Margin::ZERO;

    pub const fn all(v: f32) -> Margin {
        Margin { l: v, t: v, r: v, b: v }
    }

    pub const fn left(v: f32) -> Margin {
        Margin { l: v, ..Margin::ZERO }
    }

    pub const fn top(v: f32) -> Margin {
        Margin { t: v, ..Margin::ZERO }
    }

    pub const fn right(v: f32) -> Margin {
        Margin { r: v, ..Margin::ZERO }
    }

    pub const fn bottom(v: f32) -> Margin {
        Margin { b: v, ..Margin::ZERO }
    }
}

impl Default for Margin {
    fn default() -> Self {
        Margin::DEFAULT
    }
}

/// Modify the behavior of [`Event::hits`].
#[derive(Clone, Debug, Default)]
pub struct HitOpt {
    pub use_multi_touch: bool,
    pub margin: Option<Margin>,
}

impl Event {
    /// Checks if an [`Event`] matches an [`ComponentBase`].
    ///
    /// For key/text events, it checks if the given [`Area`] matches _exactly_
    /// the one currently set in [`Cx::key_focus`] (which gets set through
    /// [`Cx::set_key_focus`]).
    ///
    /// For mouse events, it checks if the event coordinates fall in the [`Rect`]
    /// of the _first instance_ of the [`Area`].
    ///
    /// This will return [`Event::None`] if [`Area::is_valid`] returns false.
    /// This is sometimes useful, e.g. when clicking a button that brings a
    /// previously hidden [`Area`] into view. A subsequent event might then
    /// call [`Event::hits`] on that (old) [`Area`], before it had a chance
    /// to get rerendered, so we just ignore it.
    ///
    /// TODO(JP): For the [`Area::is_valid`] case described above, should we
    /// instead explicitly set an area to [`Area::Empty`] when it goes out of
    /// view? Would that be better, or just more work?
    ///
    /// TODO(JP): Only checking the "first instance" for mouse events is
    /// confusing. Ideally we would check if the event falls within any of the
    /// instances covered by the [`Area`].
    pub fn hits(&mut self, cx: &mut Cx, component_base: &ComponentBase, opt: HitOpt) -> Event {
        let area = component_base.area;
        let id = component_base.id;

        if area.is_empty() || !area.is_valid(cx) {
            return Event::None;
        }
        match self {
            Event::KeyFocus(kf) => {
                if id == kf.prev {
                    return Event::KeyFocusLost(kf.clone());
                } else if id == kf.focus {
                    return Event::KeyFocus(kf.clone());
                }
            }
            Event::KeyDown(_) => {
                if id == cx.key_focus {
                    return self.clone();
                }
            }
            Event::KeyUp(_) => {
                if id == cx.key_focus {
                    return self.clone();
                }
            }
            Event::TextInput(_) => {
                if id == cx.key_focus {
                    return self.clone();
                }
            }
            Event::TextCopy => {
                if id == cx.key_focus {
                    return Event::TextCopy;
                }
            }
            Event::FingerScroll(fe) => {
                let rect = area.get_rect_for_first_instance(cx);
                if rect_contains_with_margin(&rect, fe.abs, &opt.margin) {
                    //fe.handled = true;
                    return Event::FingerScroll(FingerScrollEvent { rel: fe.abs - rect.pos, rect, ..fe.clone() });
                }
            }
            Event::FingerHover(fe) => {
                let rect = area.get_rect_for_first_instance(cx);

                if cx.fingers[fe.digit]._over_last == id {
                    let mut any_down = false;
                    for finger in &cx.fingers {
                        if finger.captured == id {
                            any_down = true;
                            break;
                        }
                    }
                    if !fe.handled && rect_contains_with_margin(&rect, fe.abs, &opt.margin) {
                        fe.handled = true;
                        if let HoverState::Out = fe.hover_state {
                            //    cx.finger_over_last_area = Area::Empty;
                        } else {
                            cx.fingers[fe.digit].over_last = id;
                        }
                        return Event::FingerHover(FingerHoverEvent { rel: fe.abs - rect.pos, rect, any_down, ..fe.clone() });
                    } else {
                        //self.was_over_last_call = false;
                        return Event::FingerHover(FingerHoverEvent {
                            rel: fe.abs - rect.pos,
                            rect,
                            any_down,
                            hover_state: HoverState::Out,
                            ..fe.clone()
                        });
                    }
                } else if !fe.handled && rect_contains_with_margin(&rect, fe.abs, &opt.margin) {
                    let mut any_down = false;
                    for finger in &cx.fingers {
                        if finger.captured == id {
                            any_down = true;
                            break;
                        }
                    }
                    cx.fingers[fe.digit].over_last = id;
                    fe.handled = true;
                    //self.was_over_last_call = true;
                    return Event::FingerHover(FingerHoverEvent {
                        rel: fe.abs - rect.pos,
                        rect,
                        any_down,
                        hover_state: HoverState::In,
                        ..fe.clone()
                    });
                }
            }
            Event::FingerMove(fe) => {
                // check wether our digit is captured, otherwise don't send
                if cx.fingers[fe.digit].captured == id {
                    let abs_start = cx.fingers[fe.digit].down_abs_start;
                    let rel_start = cx.fingers[fe.digit].down_rel_start;
                    let rect = area.get_rect_for_first_instance(cx);
                    return Event::FingerMove(FingerMoveEvent {
                        abs_start,
                        rel: fe.abs - rect.pos,
                        rel_start,
                        rect,
                        is_over: rect_contains_with_margin(&rect, fe.abs, &opt.margin),
                        ..fe.clone()
                    });
                }
            }
            Event::FingerDown(fe) => {
                if !fe.handled {
                    let rect = area.get_rect_for_first_instance(cx);
                    if rect_contains_with_margin(&rect, fe.abs, &opt.margin) {
                        // scan if any of the fingers already captured this area
                        if !opt.use_multi_touch {
                            for finger in &cx.fingers {
                                if finger.captured == id {
                                    return Event::None;
                                }
                            }
                        }
                        cx.fingers[fe.digit].captured = id;
                        let rel = fe.abs - rect.pos;
                        cx.fingers[fe.digit].down_abs_start = fe.abs;
                        cx.fingers[fe.digit].down_rel_start = rel;
                        fe.handled = true;
                        return Event::FingerDown(FingerDownEvent { rel, rect, ..fe.clone() });
                    }
                }
            }
            Event::FingerUp(fe) => {
                if cx.fingers[fe.digit].captured == id {
                    cx.fingers[fe.digit].captured = None;
                    let abs_start = cx.fingers[fe.digit].down_abs_start;
                    let rel_start = cx.fingers[fe.digit].down_rel_start;
                    let rect = area.get_rect_for_first_instance(cx);
                    return Event::FingerUp(FingerUpEvent {
                        is_over: rect.contains(fe.abs),
                        abs_start,
                        rel_start,
                        rel: fe.abs - rect.pos,
                        rect,
                        ..fe.clone()
                    });
                }
            }
            _ => (),
        };
        Event::None
    }
}

fn rect_contains_with_margin(rect: &Rect, pos: Vec2, margin: &Option<Margin>) -> bool {
    if let Some(margin) = margin {
        pos.x >= rect.pos.x - margin.l
            && pos.x <= rect.pos.x + rect.size.x + margin.r
            && pos.y >= rect.pos.y - margin.t
            && pos.y <= rect.pos.y + rect.size.y + margin.b
    } else {
        rect.contains(pos)
    }
}

/// For firing and capturing custom events. Can even be fired from different
/// threads using [`Cx::post_signal`].
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, Default)]
pub struct Signal {
    pub signal_id: usize,
}

/// Status field to send with a [`Signal`].
///
/// An alias over LocationHash so we have a semantic type
/// but can change the underlying implementation whenever.
pub type StatusId = LocationHash;

/// Created using [`Cx::start_timer`].
#[derive(Clone, Debug, Default)]
pub struct Timer {
    pub timer_id: u64,
}

impl Timer {
    pub fn empty() -> Timer {
        Timer { timer_id: 0 }
    }

    pub fn is_timer(&mut self, te: &TimerEvent) -> bool {
        te.timer_id == self.timer_id
    }
}

/// Lowest common denominator keymap between desktop and web.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KeyCode {
    Escape,

    Backtick,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Minus,
    Equals,

    Backspace,
    Tab,

    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    LBracket,
    RBracket,
    Return,

    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    Semicolon,
    Quote,
    Backslash,

    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Comma,
    Period,
    Slash,

    Control,
    Alt,
    Shift,
    Logo,

    //RightControl,
    //RightShift,
    //RightAlt,
    //RightLogo,
    Space,
    Capslock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    PrintScreen,
    Scrolllock,
    Pause,

    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,

    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

    NumpadEquals,
    NumpadSubtract,
    NumpadAdd,
    NumpadDecimal,
    NumpadMultiply,
    NumpadDivide,
    Numlock,
    NumpadEnter,

    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    Unknown,
}

impl Default for KeyCode {
    fn default() -> Self {
        KeyCode::Unknown
    }
}
