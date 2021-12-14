// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::cx::*;
use crate::zerde::*;

pub(crate) fn parse_keyboard_event_from_js(msg_type: u32, zerde_parser: &mut ZerdeParser) -> Event {
    match msg_type {
        12 => {
            // key_down
            Event::KeyDown(KeyEvent {
                key_code: parse_key_code_from_web(zerde_parser.parse_u32()),
                is_repeat: zerde_parser.parse_u32() > 0,
                modifiers: unpack_key_modifier(zerde_parser.parse_u32()),
                time: zerde_parser.parse_f64(),
            })
        }
        13 => {
            // key_up
            Event::KeyUp(KeyEvent {
                key_code: parse_key_code_from_web(zerde_parser.parse_u32()),
                is_repeat: zerde_parser.parse_u32() > 0,
                modifiers: unpack_key_modifier(zerde_parser.parse_u32()),
                time: zerde_parser.parse_f64(),
            })
        }
        14 => {
            // text_input
            Event::TextInput(TextInputEvent {
                was_paste: zerde_parser.parse_u32() > 0,
                replace_last: zerde_parser.parse_u32() > 0,
                input: zerde_parser.parse_string(),
            })
        }
        17 => {
            // text_copy
            Event::TextCopy
        }
        _ => {
            panic!("Message unknown {}", msg_type);
        }
    }
}

pub(crate) fn unpack_key_modifier(modifiers: u32) -> KeyModifiers {
    KeyModifiers {
        shift: (modifiers & 1) != 0,
        control: (modifiers & 2) != 0,
        alt: (modifiers & 4) != 0,
        logo: (modifiers & 8) != 0,
    }
}

fn parse_key_code_from_web(key_code: u32) -> KeyCode {
    match key_code {
        27 => KeyCode::Escape,
        192 => KeyCode::Backtick,
        48 => KeyCode::Key0,
        49 => KeyCode::Key1,
        50 => KeyCode::Key2,
        51 => KeyCode::Key3,
        52 => KeyCode::Key4,
        53 => KeyCode::Key5,
        54 => KeyCode::Key6,
        55 => KeyCode::Key7,
        56 => KeyCode::Key8,
        57 => KeyCode::Key9,
        173 => KeyCode::Minus,
        189 => KeyCode::Minus,
        61 => KeyCode::Equals,
        187 => KeyCode::Equals,

        8 => KeyCode::Backspace,
        9 => KeyCode::Tab,

        81 => KeyCode::KeyQ,
        87 => KeyCode::KeyW,
        69 => KeyCode::KeyE,
        82 => KeyCode::KeyR,
        84 => KeyCode::KeyT,
        89 => KeyCode::KeyY,
        85 => KeyCode::KeyU,
        73 => KeyCode::KeyI,
        79 => KeyCode::KeyO,
        80 => KeyCode::KeyP,
        219 => KeyCode::LBracket,
        221 => KeyCode::RBracket,
        13 => KeyCode::Return,

        65 => KeyCode::KeyA,
        83 => KeyCode::KeyS,
        68 => KeyCode::KeyD,
        70 => KeyCode::KeyF,
        71 => KeyCode::KeyG,
        72 => KeyCode::KeyH,
        74 => KeyCode::KeyJ,
        75 => KeyCode::KeyK,
        76 => KeyCode::KeyL,

        59 => KeyCode::Semicolon,
        186 => KeyCode::Semicolon,
        222 => KeyCode::Quote,
        220 => KeyCode::Backslash,

        90 => KeyCode::KeyZ,
        88 => KeyCode::KeyX,
        67 => KeyCode::KeyC,
        86 => KeyCode::KeyV,
        66 => KeyCode::KeyB,
        78 => KeyCode::KeyN,
        77 => KeyCode::KeyM,
        188 => KeyCode::Comma,
        190 => KeyCode::Period,
        191 => KeyCode::Slash,

        17 => KeyCode::Control,
        18 => KeyCode::Alt,
        16 => KeyCode::Shift,
        224 => KeyCode::Logo,
        91 => KeyCode::Logo,

        //RightControl,
        //RightShift,
        //RightAlt,
        93 => KeyCode::Logo,

        32 => KeyCode::Space,
        20 => KeyCode::Capslock,
        112 => KeyCode::F1,
        113 => KeyCode::F2,
        114 => KeyCode::F3,
        115 => KeyCode::F4,
        116 => KeyCode::F5,
        117 => KeyCode::F6,
        118 => KeyCode::F7,
        119 => KeyCode::F8,
        120 => KeyCode::F9,
        121 => KeyCode::F10,
        122 => KeyCode::F11,
        123 => KeyCode::F12,

        44 => KeyCode::PrintScreen,
        124 => KeyCode::PrintScreen,
        //Scrolllock,
        //Pause,
        45 => KeyCode::Insert,
        46 => KeyCode::Delete,
        36 => KeyCode::Home,
        35 => KeyCode::End,
        33 => KeyCode::PageUp,
        34 => KeyCode::PageDown,

        96 => KeyCode::Numpad0,
        97 => KeyCode::Numpad1,
        98 => KeyCode::Numpad2,
        99 => KeyCode::Numpad3,
        100 => KeyCode::Numpad4,
        101 => KeyCode::Numpad5,
        102 => KeyCode::Numpad6,
        103 => KeyCode::Numpad7,
        104 => KeyCode::Numpad8,
        105 => KeyCode::Numpad9,

        //NumpadEquals,
        109 => KeyCode::NumpadSubtract,
        107 => KeyCode::NumpadAdd,
        110 => KeyCode::NumpadDecimal,
        106 => KeyCode::NumpadMultiply,
        111 => KeyCode::NumpadDivide,
        12 => KeyCode::Numlock,
        //NumpadEnter,
        38 => KeyCode::ArrowUp,
        40 => KeyCode::ArrowDown,
        37 => KeyCode::ArrowLeft,
        39 => KeyCode::ArrowRight,
        _ => KeyCode::Unknown,
    }
}
