use sdl3::keyboard::Keycode;

#[derive(Default)]
pub struct Input {
    pub mouse: (i32, i32),
    pub click: Option<(i32, i32)>,
    pub clicks: u8,
    pub mouse_down: bool,
    pub scroll_px: f32,
    pub text: String,
    pub keys: Vec<Keycode>,
    pub shift: bool,
    pub ctrl: bool,
    pub caret_on: bool,
}
