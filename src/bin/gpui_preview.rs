#[path = "../focus.rs"]
mod focus;
#[path = "../gpui_access.rs"]
mod gpui_access;
#[path = "../gpui_input.rs"]
mod gpui_input;
#[path = "../gpui_ui.rs"]
mod gpui_ui;
#[path = "../patient.rs"]
mod patient;
#[path = "../storage/mod.rs"]
mod storage;

use gpui::{
    App, AppContext, Bounds, KeyBinding, Point, QuitMode, Size, TitlebarOptions, WindowBounds,
    WindowOptions, actions, px,
};
use gpui_platform::application;

actions!(gpui_preview, [Quit]);

fn main() {
    application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(|cx: &mut App| {
            cx.bind_keys([
                KeyBinding::new("cmd-q", Quit, None),
                KeyBinding::new("backspace", gpui_input::Backspace, Some("TextArea")),
                KeyBinding::new("delete", gpui_input::Delete, Some("TextArea")),
                KeyBinding::new("left", gpui_input::Left, Some("TextArea")),
                KeyBinding::new("right", gpui_input::Right, Some("TextArea")),
                KeyBinding::new("home", gpui_input::Home, Some("TextArea")),
                KeyBinding::new("end", gpui_input::End, Some("TextArea")),
                KeyBinding::new("up", gpui_input::Up, Some("TextArea")),
                KeyBinding::new("down", gpui_input::Down, Some("TextArea")),
                KeyBinding::new("shift-left", gpui_input::SelectLeft, Some("TextArea")),
                KeyBinding::new("shift-right", gpui_input::SelectRight, Some("TextArea")),
                KeyBinding::new("alt-left", gpui_input::WordLeft, Some("TextArea")),
                KeyBinding::new("ctrl-left", gpui_input::WordLeft, Some("TextArea")),
                KeyBinding::new("alt-right", gpui_input::WordRight, Some("TextArea")),
                KeyBinding::new("ctrl-right", gpui_input::WordRight, Some("TextArea")),
                KeyBinding::new("backspace", gpui_input::Backspace, Some("TextInput")),
                KeyBinding::new("delete", gpui_input::Delete, Some("TextInput")),
                KeyBinding::new("left", gpui_input::Left, Some("TextInput")),
                KeyBinding::new("right", gpui_input::Right, Some("TextInput")),
                KeyBinding::new("home", gpui_input::Home, Some("TextInput")),
                KeyBinding::new("end", gpui_input::End, Some("TextInput")),
                KeyBinding::new("shift-left", gpui_input::SelectLeft, Some("TextInput")),
                KeyBinding::new("shift-right", gpui_input::SelectRight, Some("TextInput")),
                KeyBinding::new("cmd-a", gpui_input::SelectAll, Some("TextInput")),
                KeyBinding::new("ctrl-a", gpui_input::SelectAll, Some("TextInput")),
                KeyBinding::new("cmd-c", gpui_input::Copy, Some("TextInput")),
                KeyBinding::new("ctrl-c", gpui_input::Copy, Some("TextInput")),
                KeyBinding::new("cmd-x", gpui_input::Cut, Some("TextInput")),
                KeyBinding::new("ctrl-x", gpui_input::Cut, Some("TextInput")),
                KeyBinding::new("cmd-v", gpui_input::Paste, Some("TextInput")),
                KeyBinding::new("ctrl-v", gpui_input::Paste, Some("TextInput")),
                KeyBinding::new("cmd-z", gpui_input::Undo, Some("TextInput")),
                KeyBinding::new("ctrl-z", gpui_input::Undo, Some("TextInput")),
                KeyBinding::new("cmd-shift-z", gpui_input::Redo, Some("TextInput")),
                KeyBinding::new("ctrl-y", gpui_input::Redo, Some("TextInput")),
                KeyBinding::new("alt-left", gpui_input::WordLeft, Some("TextInput")),
                KeyBinding::new("ctrl-left", gpui_input::WordLeft, Some("TextInput")),
                KeyBinding::new("alt-right", gpui_input::WordRight, Some("TextInput")),
                KeyBinding::new("ctrl-right", gpui_input::WordRight, Some("TextInput")),
                KeyBinding::new("cmd-a", gpui_input::SelectAll, Some("TextArea")),
                KeyBinding::new("ctrl-a", gpui_input::SelectAll, Some("TextArea")),
                KeyBinding::new("cmd-c", gpui_input::Copy, Some("TextArea")),
                KeyBinding::new("ctrl-c", gpui_input::Copy, Some("TextArea")),
                KeyBinding::new("cmd-x", gpui_input::Cut, Some("TextArea")),
                KeyBinding::new("ctrl-x", gpui_input::Cut, Some("TextArea")),
                KeyBinding::new("cmd-v", gpui_input::Paste, Some("TextArea")),
                KeyBinding::new("ctrl-v", gpui_input::Paste, Some("TextArea")),
                KeyBinding::new("cmd-z", gpui_input::Undo, Some("TextArea")),
                KeyBinding::new("ctrl-z", gpui_input::Undo, Some("TextArea")),
                KeyBinding::new("cmd-shift-z", gpui_input::Redo, Some("TextArea")),
                KeyBinding::new("ctrl-y", gpui_input::Redo, Some("TextArea")),
                KeyBinding::new("enter", gpui_access::Submit, Some("TextInput")),
                KeyBinding::new("enter", gpui_input::Newline, Some("TextArea")),
                KeyBinding::new("tab", gpui_access::NextField, None),
                KeyBinding::new("shift-tab", gpui_access::PreviousField, None),
            ]);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: Size {
                        width: px(980.),
                        height: px(840.),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("KinéBilan 0.8.2 — GPUI".into()),
                    ..Default::default()
                }),
                ..WindowOptions::default()
            };
            cx.open_window(options, |_, cx| cx.new(gpui_access::AccessView::new))
                .unwrap();
            cx.on_action(|_: &Quit, cx| cx.quit());
            cx.activate(true);
        });
}
