// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;
use wrflib_widget::*;

#[derive(Default)]
pub struct SingleButton {
    button: Button,

    clicked: bool,
}

impl SingleButton {
    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let ButtonEvent::Clicked = self.button.handle(cx, event) {
            self.clicked = true;
            cx.request_draw();
        }
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.button.draw(cx, if self.clicked { "Hello world!" } else { "Click me!" });
    }
}

/*

Equivalent React component-style:

class SingleButton {
    state = {
        clicked: false,
    },

    onButtonClick = () => {
        this.setState({ clicked: true });
    }

    render() {
        return (
            <button onClick={this.onButtonClick}>
              {this.clicked ? "Hello world!" : "Click me!"}
            </button>
        );
    }
}

Equivalent React functional-style:

function SingleButton() {
    const [clicked, setClicked] = useState(false);

    const onButtonClick = useCallback(() => {
        setClicked(true);
    }, [setClicked]);

    return (
        <button onClick={onButtonClick}>
            {clicked ? "Hello world!" : "Click me!"}
        </button>
    );
}

*/
