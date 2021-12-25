// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;
use wrflib_widget::*;

pub struct HomePage {
    view: ScrollView,
    email_input: TextInput,
    email_state: EmailState,
    email_signal: Signal,
    send_mail_button: NormalButton,
}

#[derive(Clone)]
enum EmailState {
    Empty,
    Invalid,
    Valid,
    Sending,
    ErrorSending,
    OkSending,
}

const TEXT_STYLE_HEADING: TextStyle = TextStyle { font_size: 28.0, line_spacing: 2.0, ..TEXT_STYLE_NORMAL };
const TEXT_STYLE_BODY: TextStyle = TextStyle { font_size: 10.0, height_factor: 2.0, line_spacing: 3.0, ..TEXT_STYLE_NORMAL };
const TEXT_COLOR: Vec4 = vec4(187.0 / 255.0, 187.0 / 255.0, 187.0 / 255.0, 1.0);

const TEXT_HEADING: TextInsProps = TextInsProps {
    text_style: TEXT_STYLE_HEADING,
    wrapping: Wrapping::Word,
    color: TEXT_COLOR,
    padding: Padding { l: 0., r: 0., t: 10., b: 10. },
    ..TextInsProps::DEFAULT
};
const TEXT_BODY: TextInsProps = TextInsProps {
    text_style: TEXT_STYLE_BODY,
    wrapping: Wrapping::Word,
    color: TEXT_COLOR,
    padding: Padding { l: 0., r: 0., t: 10., b: 10. },
    ..TextInsProps::DEFAULT
};

impl HomePage {
    pub fn new(cx: &mut Cx) -> Self {
        Self {
            view: ScrollView::new_standard_hv(),
            send_mail_button: NormalButton::default(),
            email_signal: cx.new_signal(),
            email_input: TextInput::new(TextInputOptions {
                multiline: false,
                read_only: false,
                empty_message: "Enter email".to_string(),
            }),
            email_state: EmailState::Empty,
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let Event::Signal(sig) = event {
            if let Some(statusses) = sig.signals.get(&self.email_signal) {
                for status in statusses {
                    if *status == Cx::STATUS_HTTP_SEND_OK {
                        self.email_state = EmailState::OkSending;
                    } else if *status == Cx::STATUS_HTTP_SEND_FAIL {
                        self.email_state = EmailState::ErrorSending;
                    }
                    cx.request_draw();
                }
            }
        }
        if let TextEditorEvent::Change = self.email_input.handle(cx, event) {
            let email = self.email_input.get_value();

            if !email.is_empty() && email.find('@').is_none() {
                self.email_state = EmailState::Invalid
            } else if !email.is_empty() {
                self.email_state = EmailState::Valid
            } else {
                self.email_state = EmailState::Empty
            }
            cx.request_draw();
        }

        if let ButtonEvent::Clicked = self.send_mail_button.handle(cx, event) {
            match self.email_state {
                EmailState::Valid | EmailState::ErrorSending => {
                    self.email_state = EmailState::Sending;
                    cx.request_draw();
                }
                _ => (),
            }
        }

        self.view.handle(cx, event);
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.view.begin_view(cx, Layout::default());
        let text_turtle = cx.begin_turtle(Layout {
            padding: Padding { l: 10., r: 10., t: 0., b: 0. },
            direction: Direction::Down,
            walk: Walk { width: Width::FillUntil(550.), height: Height::Compute, ..Walk::default() },
            ..Layout::default()
        });

        TextIns::draw_walk(cx, "Introducing Bigedit\n", &TEXT_HEADING);

        TextIns::draw_walk(
            cx,
            "\
            Bigedit is an example application for Wrf. It's the original Makepad editor, but with a lot of features removed. \
             It's mostly used as an example to make sure you don't break stuff when you edit the framework code.\n",
            &TEXT_BODY,
        );

        let row = cx.begin_row_turtle();
        self.email_input.draw(cx);
        self.send_mail_button.draw(
            cx,
            match self.email_state {
                EmailState::Empty => "Sign up for our newsletter here.",
                EmailState::Invalid => "Email adress invalid",
                EmailState::Valid => "Click here to subscribe to our newsletter",
                EmailState::Sending => "Submitting your email adress..",
                EmailState::ErrorSending => "Could not send your email adress, please retry!",
                EmailState::OkSending => "Thank you, we'll keep you informed!",
            },
        );
        cx.end_turtle(row);

        TextIns::draw_walk(cx, "Lorem ipsum, etcetera! :-) \n", &TEXT_BODY);

        TextIns::draw_walk(cx, "A nice little heading\n", &TEXT_HEADING);

        TextIns::draw_walk(
            cx,
            "\
            On all platforms first install Rust. On windows feel free to ignore the warnings about MSVC, Bigedit uses the gnu \
             chain. Copy this url to your favorite browser.\n",
            &TEXT_BODY,
        );

        TextIns::draw_walk(cx, "Lorem Ipsum\n", &TEXT_HEADING);

        TextIns::draw_walk(
            cx,
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean dictum consectetur eros, vitae interdum enim \
             accumsan eu. Vivamus et erat ornare, tristique massa quis, tincidunt felis. Sed vel massa sed tellus efficitur \
             congue id ut elit. Nullam tempus vestibulum ante ut pharetra. Proin eget ex nisl. Vivamus ornare malesuada metus. \
             Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Vivamus nunc mi, tincidunt \
             non lorem at, ultrices facilisis dolor. Duis non augue ac sapien dapibus consequat. Morbi a velit a leo egestas \
             consectetur. Proin auctor purus quis dignissim interdum. Proin gravida leo mi, non rhoncus neque efficitur nec. In \
             hac habitasse platea dictumst. Nulla quis auctor ante, et tincidunt sem.\n",
            &TEXT_BODY,
        );

        cx.end_turtle(text_turtle);

        ScrollShadow::draw_shadow_top(cx, 10.0);

        self.view.end_view(cx);
    }
}
