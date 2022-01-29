// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;

#[derive(Clone, PartialEq)]
pub enum ButtonLogicEvent {
    Over,
    Default,
    Down,
}

#[derive(Clone, PartialEq)]
pub enum ButtonEvent {
    None,
    Clicked,
    Down,
    Up,
}

pub fn handle_button_logic<F>(cx: &mut Cx, hit_event: Event, mut cb: F) -> ButtonEvent
where
    F: FnMut(&mut Cx, ButtonLogicEvent),
{
    match hit_event {
        Event::FingerDown(_fe) => {
            cb(cx, ButtonLogicEvent::Down);
            return ButtonEvent::Down;
        }
        Event::FingerHover(fe) => {
            cx.set_hover_mouse_cursor(MouseCursor::Hand);
            match fe.hover_state {
                HoverState::In => {
                    if fe.any_down {
                        cb(cx, ButtonLogicEvent::Down);
                    } else {
                        cb(cx, ButtonLogicEvent::Over);
                    }
                }
                HoverState::Out => cb(cx, ButtonLogicEvent::Default),
                _ => (),
            }
        }
        Event::FingerUp(fe) => {
            if fe.is_over {
                if fe.input_type.has_hovers() {
                    cb(cx, ButtonLogicEvent::Over)
                } else {
                    cb(cx, ButtonLogicEvent::Default)
                }
                return ButtonEvent::Clicked;
            } else {
                cb(cx, ButtonLogicEvent::Default);
                return ButtonEvent::Up;
            }
        }
        _ => (),
    };
    ButtonEvent::None
}
