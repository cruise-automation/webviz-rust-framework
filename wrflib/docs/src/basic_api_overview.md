# API Overview

## Universal APIs

For the most part, you can use normal Rust APIs. However, some standard Rust APIs don't work with WebAssembly, so we've built our own cross-platform "universal" APIs.

| Rust API | Universal API | |
|----------|---------------|-------|
| [`println!`](https://doc.rust-lang.org/std/macro.println.html) | [`log!`](/target/doc/wrflib/macro.log.html) | Logs to the console (with line number). |
| [`thread`](https://doc.rust-lang.org/std/thread/) | [`universal_thread`](/target/doc/wrflib/universal_thread/index.html) | <ul><li><code><a href="/target/doc/wrflib/universal_thread/fn.spawn.html">spawn</a></code> (without <code><a href="https://doc.rust-lang.org/std/thread/struct.JoinHandle.html">JoinHandle</a></code>)</li><li><code><a href="/target/doc/wrflib/universal_thread/fn.sleep.html">sleep</a></code></li><li>We recommend using a thread pool, e.g. the <a href="https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html#method.spawn_handler">rayon crate's <code>ThreadPoolBuilder</code></a>.</li></ul> |
| [`Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html) | [`UniversalInstant`](/target/doc/wrflib/universal_instant/struct.UniversalInstant.html) | `elapsed, now, duration_since, checked_add, checked_sub, +, -, +=, -=` |
| [`File`](https://doc.rust-lang.org/std/thread/) | [`UniversalFile`](/target/doc/wrflib/universal_file/struct.UniversalFile.html) | <ul><li><code><a href="/target/doc/wrflib/universal_file/struct.UniversalFile.html#method.open">open</a></code> (on WebAssembly this blocks until the whole file is loaded in memory)</li><li><code><a href="/target/doc/wrflib/universal_file/struct.UniversalFile.html#method.open_url">open_url</a></code> (non-standard; load an absolute URL)</li><li><code><a href="/target/doc/wrflib/universal_file/struct.UniversalFile.html#method.clone">clone</a></code> (cheap; clones just a handle to the data; doesn't preserve cursor)</li><li><code><a href="https://doc.rust-lang.org/std/io/trait.Read.html">std::io::Read</a></code></li><li><code><a href="https://doc.rust-lang.org/std/io/trait.Seek.html">std::io::Seek</a></code></li><li><code><a href="/target/doc/wrflib/read_seek/trait.ReadSeek.html">ReadSeek</a></code> (non-standard; convenient trait for <code>Read + Seek</code>)</li></ul> |
| non-standard | [`universal_http_stream`](/target/doc/wrflib/universal_http_stream/index.html) | <ul><li><code><a href="/target/doc/wrflib/universal_http_stream/fn.request.html">request</a></code> (returns data as it comes in; useful for large files)</li><li><code><a href="https://doc.rust-lang.org/std/io/trait.Read.html">std::io::Read</a></code></li></ul> |
| non-standard | [`universal_rand`](/target/doc/wrflib/universal_rand/index.html) | [`random_128`](/target/doc/wrflib/universal_rand/fn.random_128.html) |

## `Cx` & basic events

As you might have seen in [Tutorial: Hello World Console](./tutorial_hello_world_console.md), we can get events in our application.

We also have access to a [`Cx`](/target/doc/wrflib/cx/struct.Cx.html) object. This is a global "context" object, that gets passed around practically everywhere.

Here we'll look at the basic calls you can make on `Cx`, and their associated events. We'll save rendering-related calls and events for a later chapter.

### Construction

When the app is constructed and APIs can be called, a [`Construct`](/target/doc/wrflib/enum.Event.html#variant.Construct) event is fired. It is fired exactly once, and before any other calls to `handle` or `draw`. The event contains no futher information.

### Timers

Calling [`cx.start_timer`](/target/doc/wrflib/struct.Cx.html#method.start_timer) creates a new [`Timer`](/target/doc/wrflib/struct.Timer.html) object. When the timer fires, a [`TimerEvent`](/target/doc/wrflib/struct.TimerEvent.html) event is dispatched. Use [`timer.is_timer`](/target/doc/wrflib/struct.Timer.html#method.is_timer) to check if that event belongs to a particular timer. Use [`cx.stop_timer`](/target/doc/wrflib/struct.Cx.html#method.stop_timer) to stop it.

### Signals

Signals are user-defined events that can be used for anything you want. Create a new [`Signal`](/target/doc/wrflib/struct.Signal.html) object by calling [`cx.new_signal`](/target/doc/wrflib/struct.Cx.html#method.new_signal). Then send it with a [`StatusId`](/target/doc/wrflib/type.StatusId.html) using [`cx.send_signal`](/target/doc/wrflib/struct.Cx.html#method.send_signal) (same thread) or [`Cx::post_signal`](/target/doc/wrflib/struct.Cx.html#method.post_signal) (any thread). This will trigger a [`SignalEvent`](/target/doc/wrflib/struct.SignalEvent.html) on the main thread (`handle` and `draw` are always called on the main thread).

Note that the Signals API is a bit complicated currently; we aim to improve this so you can send any user-defined events.

### WebSockets

[`cx.websocket_send`](/target/doc/wrflib/struct.Cx.html#method.websocket_send) sends a message on a WebSocket. If no WebSocket yet exists for the given URL, a new one is opened. When receiving a message on a WebSocket, a [WebSocketMessageEvent](/target/doc/wrflib/struct.WebSocketMessageEvent.html) is fired.

### Focus

If the browser tab or native window gets or loses focus, then [`AppFocus`](/target/doc/wrflib/enum.Event.html#variant.AppFocus) or [`AppFocusLost`](/target/doc/wrflib/enum.Event.html#variant.AppFocusLost) are fired respectively.

### User files

This is getting a bit into rendering territory, since we already showed this in a [tutorial](./tutorial_hello_thread.md#drag--drop-files), we'll cover it here. To create a drop target for the entire window / browser tab, we have to create a [`Window`](/target/doc/wrflib/struct.Window.html) with [`create_add_drop_target_for_app_open_files`](/target/doc/wrflib/struct.Window.html#structfield.create_add_drop_target_for_app_open_files). Then, when dropping a file, an [`AppOpenFilesEvent`](/target/doc/wrflib/struct.AppOpenFilesEvent.html) event will fire.

There are also events for when a file drag is [started](/target/doc/wrflib/enum.Event.html#variant.FileDragBegin), [updated](/target/doc/wrflib/enum.Event.html#variant.FileDragUpdate), or [cancelled](/target/doc/wrflib/enum.Event.html#variant.FileDragCancel).

### Profiling

Basic profiling using the console can be done using [`cx.profile_start`](/target/doc/wrflib/struct.Cx.html#method.profile_start) and [`cx.profile_end`](/target/doc/wrflib/struct.Cx.html#method.profile_end).
