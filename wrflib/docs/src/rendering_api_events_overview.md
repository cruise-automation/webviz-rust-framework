# Events

`fn handle()` is the application entrypoint for handling events, and is passed an [`Event`](/target/doc/wrflib/enum.Event.html). For a detailed view, you can read all of the variants of the `Event` enum. We'll outline different event scenarios below.

## Structuring the `handle` function
Usually, both the top level app and individual components will all have a `handle` function that takes in an `Event`. These functions should:
 * use a `match` expression to handle relevant events for the component
 * pass the `Event` to all child components' `handle` functions
 * call [`cx.request_draw()`](/target/doc/wrflib/struct.Cx.html#method.request_draw) if a redraw is necessitated.
 * call [`cx.request_frame()`](/target/doc/wrflib/struct.Cx.html#method.request_frame) if it should trigger another call to the top level `handle`.

## User input

Mouse and touch input are called "pointers" in Wrflib, represented using [`PointerUp`](/target/doc/wrflib/enum.Event.html#variant.PointerUp), [`PointerDown`](/target/doc/wrflib/enum.Event.html#variant.PointerDown), [`PointerMove`](/target/doc/wrflib/enum.Event.html#variant.PointerMove), and [`PointerScroll`](/target/doc/wrflib/enum.Event.html#variant.PointerScroll), and [`PointerHover`](/target/doc/wrflib/enum.Event.html#variant.PointerHover).

For processing text input, use [`TextInput`](/target/doc/wrflib/enum.Event.html#variant.TextInput). We also have [`KeyDown`](/target/doc/wrflib/enum.Event.html#variant.KeyDown) and [`KeyUp`](/target/doc/wrflib/enum.Event.html#variant.KeyUp), useful for keyboard based navigation or shortcuts - but do not rely on these for capturing text input. Use [`TextCopy`](/target/doc/wrflib/enum.Event.html#variant.TextCopy) for handling clipboard requests.

You may have different components of your app which take keyboard input. To manage keyboard focus between them, use [`set_key_focus`](/target/doc/wrflib/struct.Cx.html#method.set_key_focus). This uses [`ComponentId`](http://localhost:4848/target/doc/wrflib/struct.ComponentId.html) as a unique identifier, which you should assign to your component struct with `ComponentId::default()`.

Then, to see if a keyboard event is meant for a component, use [`hits_keyboard`](/target/doc/wrflib/enum.Event.html#method.hits_keyboard), which will check key focus and skip irrelevant events. It also returns [`KeyFocus`](/target/doc/wrflib/enum.Event.html#variant.KeyFocus) and [`KeyFocusLost`](/target/doc/wrflib/enum.Event.html#variant.KeyFocusLost) if your component should handle focus changes.
