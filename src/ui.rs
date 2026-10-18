use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::render::{FRect, TextureCreator, WindowCanvas};
use sdl3::ttf::{Font, Sdl3TtfContext};
use sdl3::video::WindowContext;

use crate::focus::Field;

pub const PAD_X: i32 = 12;
pub const PAD_Y: i32 = 10;

pub const BG: Color = Color::RGB(243, 244, 246);
pub const WHITE: Color = Color::RGB(255, 255, 255);
pub const TITLE: Color = Color::RGB(17, 24, 39);
pub const TEXT: Color = Color::RGB(31, 41, 55);
pub const LABEL: Color = Color::RGB(55, 65, 81);
pub const MUTED: Color = Color::RGB(107, 114, 128);
pub const PLACEHOLDER: Color = Color::RGB(156, 163, 175);
pub const BORDER: Color = Color::RGB(229, 231, 235);
pub const DIVIDER: Color = Color::RGB(243, 244, 246);
pub const INPUT_BORDER: Color = Color::RGB(209, 213, 219);
pub const ACCENT: Color = Color::RGB(37, 99, 235);
pub const ACCENT_HOVER: Color = Color::RGB(29, 78, 216);
pub const ACCENT_SOFT: Color = Color::RGB(239, 246, 255);
pub const ACCENT_BORDER_SOFT: Color = Color::RGB(147, 197, 253);
pub const ITEM_BG: Color = Color::RGB(250, 250, 250);
pub const BOX_BG: Color = Color::RGB(250, 251, 252);
pub const SESSION_DATE: Color = Color::RGB(30, 64, 175);
pub const BADGE_BG: Color = Color::RGB(224, 242, 254);
pub const BADGE_TEXT: Color = Color::RGB(3, 105, 161);
pub const ALERT_BG: Color = Color::RGB(254, 242, 242);
pub const ALERT_BORDER: Color = Color::RGB(254, 202, 202);
pub const ALERT_TEXT: Color = Color::RGB(185, 28, 28);
pub const SECONDARY_BG: Color = Color::RGB(229, 231, 235);
pub const SECONDARY_HOVER: Color = Color::RGB(209, 213, 219);
pub const DANGER_BG: Color = Color::RGB(254, 226, 226);
pub const DANGER_HOVER: Color = Color::RGB(254, 202, 202);
pub const DANGER_BORDER: Color = Color::RGB(252, 165, 165);
pub const DANGER_STRONG: Color = Color::RGB(220, 38, 38);
pub const BACKDROP: Color = Color::RGBA(0, 0, 0, 102);

const REGULAR_FONTS: &[&str] = &[
    r"C:\Windows\Fonts\segoeui.ttf",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/Library/Fonts/Arial.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
];

const BOLD_FONTS: &[&str] = &[
    r"C:\Windows\Fonts\segoeuib.ttf",
    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    "/Library/Fonts/Arial Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
];

pub struct Fonts {
    pub title: Font<'static>,
    pub heading: Font<'static>,
    pub section: Font<'static>,
    pub modal_title: Font<'static>,
    pub label: Font<'static>,
    pub body: Font<'static>,
    pub body_large: Font<'static>,
    pub small: Font<'static>,
    pub badge: Font<'static>,
    pub button: Font<'static>,
    pub button_large: Font<'static>,
}

impl Fonts {
    pub fn load(ttf: &Sdl3TtfContext) -> Result<Self, String> {
        Ok(Self {
            title: load_font(ttf, BOLD_FONTS, 26.0)?,
            heading: load_font(ttf, BOLD_FONTS, 20.0)?,
            section: load_font(ttf, BOLD_FONTS, 16.0)?,
            modal_title: load_font(ttf, BOLD_FONTS, 17.0)?,
            label: load_font(ttf, BOLD_FONTS, 13.0)?,
            body: load_font(ttf, REGULAR_FONTS, 14.0)?,
            body_large: load_font(ttf, REGULAR_FONTS, 15.0)?,
            small: load_font(ttf, REGULAR_FONTS, 13.0)?,
            badge: load_font(ttf, REGULAR_FONTS, 12.0)?,
            button: load_font(ttf, REGULAR_FONTS, 14.0)?,
            button_large: load_font(ttf, BOLD_FONTS, 15.0)?,
        })
    }
}

fn load_font(ttf: &Sdl3TtfContext, paths: &[&str], size: f32) -> Result<Font<'static>, String> {
    for path in paths {
        if let Ok(font) = ttf.load_font(path, size) {
            return Ok(font);
        }
    }
    Err(format!("aucune police système trouvée parmi : {}", paths.join(", ")))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(&self) -> i32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }

    pub fn inset(&self, d: i32) -> Self {
        Self::new(self.x + d, self.y + d, self.w - 2 * d, self.h - 2 * d)
    }

    pub fn contains(&self, (px, py): (i32, i32)) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    fn intersect(&self, other: Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        Rect::new(x, y, (right - x).max(0), (bottom - y).max(0))
    }
}

fn sdl_rect(r: Rect) -> sdl3::rect::Rect {
    sdl3::rect::Rect::new(r.x, r.y, r.w.max(0) as u32, r.h.max(0) as u32)
}

fn frect(r: Rect) -> FRect {
    FRect::new(r.x as f32, r.y as f32, r.w as f32, r.h as f32)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Text,
    Date,
}

impl Charset {
    fn accepts(self, c: char) -> bool {
        match self {
            Charset::Text => !c.is_control(),
            Charset::Date => c.is_ascii_digit() || c == '-',
        }
    }

    fn max_len(self) -> usize {
        match self {
            Charset::Text => usize::MAX,
            Charset::Date => 10,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
    DangerConfirm,
    Outline,
}

impl ButtonStyle {
    fn colors(self, hot: bool) -> (Color, Color, Color) {
        match self {
            ButtonStyle::Primary => {
                let bg = if hot { ACCENT_HOVER } else { ACCENT };
                (bg, WHITE, bg)
            }
            ButtonStyle::Secondary => {
                let bg = if hot { SECONDARY_HOVER } else { SECONDARY_BG };
                (bg, LABEL, bg)
            }
            ButtonStyle::Danger => {
                let bg = if hot { DANGER_HOVER } else { DANGER_BG };
                (bg, ALERT_TEXT, DANGER_BORDER)
            }
            ButtonStyle::DangerConfirm => {
                let bg = if hot { ALERT_TEXT } else { DANGER_STRONG };
                (bg, WHITE, ALERT_TEXT)
            }
            ButtonStyle::Outline => {
                let bg = if hot { ACCENT_SOFT } else { WHITE };
                (bg, ACCENT, ACCENT)
            }
        }
    }
}

#[derive(Default)]
pub struct Input {
    pub mouse: (i32, i32),
    pub click: Option<(i32, i32)>,
    pub scroll_px: f32,
    pub text: String,
    pub keys: Vec<Keycode>,
    pub shift: bool,
    pub caret_on: bool,
}

pub struct Gui<'a> {
    canvas: &'a mut WindowCanvas,
    textures: &'a TextureCreator<WindowContext>,
    pub input: Input,
    pub focus: Option<Field>,
    pub changed: bool,
    interactive: bool,
    clips: Vec<Rect>,
}

pub fn line_height(font: &Font) -> i32 {
    font.recommended_line_spacing()
}

pub fn measure(font: &Font, s: &str) -> (i32, i32) {
    font.size_of(s)
        .map(|(w, h)| (w as i32, h as i32))
        .unwrap_or((0, line_height(font)))
}

pub fn wrap(font: &Font, text: &str, max_w: i32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split(' ') {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if line.is_empty() || measure(font, &candidate).0 <= max_w {
                line = candidate;
            } else {
                lines.push(std::mem::take(&mut line));
                line = word.to_string();
            }
        }
        lines.push(line);
    }
    lines
}

pub fn area_height(font: &Font, value: &str, w: i32, min_rows: usize) -> i32 {
    let rows = if value.is_empty() {
        1
    } else {
        wrap(font, value, w - 2 * PAD_X).len()
    };
    rows.max(min_rows) as i32 * line_height(font) + 2 * PAD_Y
}

pub fn clamp_scroll(offset: i32, delta: i32, content_h: i32, viewport_h: i32) -> i32 {
    (offset + delta).clamp(0, (content_h - viewport_h).max(0))
}

impl<'a> Gui<'a> {
    pub fn new(
        canvas: &'a mut WindowCanvas,
        textures: &'a TextureCreator<WindowContext>,
        mut input: Input,
        focus: Option<Field>,
    ) -> Self {
        let mut changed = false;
        let mut focus = focus;
        if input.keys.contains(&Keycode::Tab) {
            input.keys.retain(|&k| k != Keycode::Tab);
            focus = focus.map(|f| f.tab_step(!input.shift));
            changed = true;
        }
        Self {
            canvas,
            textures,
            input,
            focus,
            changed,
            interactive: true,
            clips: Vec::new(),
        }
    }

    pub fn set_interactive(&mut self, interactive: bool) {
        self.interactive = interactive;
    }

    pub fn finish(&mut self) {
        if self.input.click.is_some() {
            self.focus = None;
            self.changed = true;
        }
    }

    pub fn fill(&mut self, rect: Rect, color: Color) {
        self.canvas.set_draw_color(color);
        let _ = self.canvas.fill_rect(frect(rect));
    }

    pub fn fill_bordered(&mut self, rect: Rect, fill: Color, border: Color) {
        self.fill(rect, fill);
        self.canvas.set_draw_color(border);
        let _ = self.canvas.draw_rect(frect(rect));
    }

    pub fn push_clip(&mut self, rect: Rect) {
        let clip = match self.clips.last() {
            Some(current) => current.intersect(rect),
            None => rect,
        };
        self.clips.push(clip);
        self.canvas.set_clip_rect(sdl_rect(clip));
    }

    pub fn pop_clip(&mut self) {
        self.clips.pop();
        match self.clips.last() {
            Some(clip) => self.canvas.set_clip_rect(sdl_rect(*clip)),
            None => self.canvas.set_clip_rect(None::<sdl3::rect::Rect>),
        }
    }

    fn in_clip(&self, point: (i32, i32)) -> bool {
        self.clips.last().is_none_or(|clip| clip.contains(point))
    }

    pub fn hovered(&self, rect: Rect) -> bool {
        self.interactive && rect.contains(self.input.mouse) && self.in_clip(self.input.mouse)
    }

    pub fn take_click(&mut self, rect: Rect) -> bool {
        let Some(point) = self.input.click else {
            return false;
        };
        if self.interactive && rect.contains(point) && self.in_clip(point) {
            self.input.click = None;
            self.changed = true;
            true
        } else {
            false
        }
    }

    pub fn take_scroll(&mut self, rect: Rect) -> i32 {
        if self.input.scroll_px == 0.0 || !self.hovered(rect) {
            return 0;
        }
        let delta = self.input.scroll_px.round() as i32;
        self.input.scroll_px = 0.0;
        self.changed = true;
        delta
    }

    pub fn scrollbar(&mut self, track: Rect, offset: i32, content_h: i32) {
        if content_h <= track.h {
            return;
        }
        let thumb_h = (track.h * track.h / content_h).max(24);
        let travel = track.h - thumb_h;
        let max_offset = content_h - track.h;
        let thumb_y = track.y + travel * offset / max_offset;
        self.fill(Rect::new(track.x, thumb_y, track.w, thumb_h), INPUT_BORDER);
    }

    pub fn text(&mut self, font: &Font, s: &str, x: i32, y: i32, color: Color) {
        if s.is_empty() {
            return;
        }
        let Ok(surface) = font.render(s).blended(color) else {
            return;
        };
        let Ok(texture) = self.textures.create_texture_from_surface(&surface) else {
            return;
        };
        let dst = FRect::new(x as f32, y as f32, surface.width() as f32, surface.height() as f32);
        let _ = self.canvas.copy(&texture, None::<FRect>, dst);
    }

    pub fn text_centered(&mut self, font: &Font, s: &str, rect: Rect, color: Color) {
        let (w, h) = measure(font, s);
        let x = rect.x + (rect.w - w) / 2;
        let y = rect.y + (rect.h - h) / 2;
        self.text(font, s, x, y, color);
    }

    pub fn button(&mut self, rect: Rect, label: &str, font: &Font, style: ButtonStyle) -> bool {
        let hot = self.hovered(rect);
        let (bg, fg, border) = style.colors(hot);
        self.fill_bordered(rect, bg, border);
        self.text_centered(font, label, rect, fg);
        self.take_click(rect)
    }

    fn edit_focus(&mut self, field: Field, rect: Rect) -> bool {
        if self.take_click(rect) {
            self.focus = Some(field);
        }
        self.focus == Some(field)
    }

    fn apply_input(&mut self, value: &mut String, multiline: bool, charset: Charset) -> bool {
        let mut changed = false;
        for c in std::mem::take(&mut self.input.text).chars() {
            if charset.accepts(c) && value.chars().count() < charset.max_len() {
                value.push(c);
                changed = true;
            }
        }
        for key in std::mem::take(&mut self.input.keys) {
            match key {
                Keycode::Backspace => changed |= value.pop().is_some(),
                Keycode::Return | Keycode::KpEnter
                    if multiline && value.chars().count() < charset.max_len() =>
                {
                    value.push('\n');
                    changed = true;
                }
                _ => {}
            }
        }
        if changed {
            self.changed = true;
        }
        changed
    }

    pub fn text_field(
        &mut self,
        field: Field,
        rect: Rect,
        value: &mut String,
        placeholder: &str,
        font: &Font,
        charset: Charset,
    ) -> bool {
        let focused = self.edit_focus(field, rect);
        let changed = focused && self.apply_input(value, false, charset);

        self.fill_bordered(rect, WHITE, if focused { ACCENT } else { INPUT_BORDER });

        let inner = Rect::new(rect.x + PAD_X, rect.y, rect.w - 2 * PAD_X, rect.h);
        let (text_w, text_h) = measure(font, value.as_str());
        let shift = (text_w - inner.w + 2).max(0);
        let text_y = rect.y + (rect.h - text_h) / 2;

        self.push_clip(inner);
        if value.is_empty() {
            self.text(font, placeholder, inner.x, text_y, PLACEHOLDER);
        } else {
            self.text(font, value, inner.x - shift, text_y, TEXT);
        }
        if focused && self.input.caret_on {
            self.fill(Rect::new(inner.x + text_w - shift, rect.y + 10, 1, rect.h - 20), TEXT);
        }
        self.pop_clip();

        changed
    }

    /// Draws a multi-line field and returns its height so the caller can lay out what follows.
    #[allow(clippy::too_many_arguments)]
    pub fn text_area(
        &mut self,
        field: Field,
        x: i32,
        y: i32,
        w: i32,
        value: &mut String,
        placeholder: &str,
        font: &Font,
        min_rows: usize,
    ) -> i32 {
        let hit = Rect::new(x, y, w, area_height(font, value, w, min_rows));
        let focused = self.edit_focus(field, hit);
        if focused {
            self.apply_input(value, true, Charset::Text);
        }

        let h = area_height(font, value, w, min_rows);
        let rect = Rect::new(x, y, w, h);
        self.fill_bordered(rect, WHITE, if focused { ACCENT } else { INPUT_BORDER });

        let lh = line_height(font);
        let text_x = x + PAD_X;
        let text_y = y + PAD_Y;
        let mut caret = (text_x, text_y);

        if value.is_empty() {
            self.text(font, placeholder, text_x, text_y, PLACEHOLDER);
        } else {
            let lines = wrap(font, value, w - 2 * PAD_X);
            for (i, line) in lines.iter().enumerate() {
                let line_y = text_y + i as i32 * lh;
                self.text(font, line, text_x, line_y, TEXT);
                caret = (text_x + measure(font, line).0, line_y);
            }
        }

        if focused && self.input.caret_on {
            self.fill(Rect::new(caret.0, caret.1, 1, lh), TEXT);
        }

        h
    }
}
