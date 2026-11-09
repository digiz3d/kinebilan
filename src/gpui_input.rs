// Text editing and IME integration adapted from GPUI-CE's view example (Apache-2.0).
use gpui::actions;

actions!(
    gpui_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Home,
        End,
        Newline,
        SelectLeft,
        SelectRight,
        SelectAll,
        Copy,
        Cut,
        Paste,
        Undo,
        Redo,
        WordLeft,
        WordRight,
        Up,
        Down
    ]
);

#[path = "gpui_input/editor.rs"]
mod editor;
#[path = "gpui_input/input.rs"]
mod input;

pub use input::Input;
