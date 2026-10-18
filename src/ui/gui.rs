use sdl3::clipboard::ClipboardUtil;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::render::{FRect, TextureCreator, WindowCanvas};
use sdl3::ttf::Font;
use sdl3::video::WindowContext;

use crate::focus::Field;

use super::caret::Caret;
use super::geometry::Rect;
use super::input::Input;
use super::text::measure;
use super::theme::INPUT_BORDER;

fn sdl_rect(r: Rect) -> sdl3::rect::Rect {
    sdl3::rect::Rect::new(r.x, r.y, r.w.max(0) as u32, r.h.max(0) as u32)
}

fn frect(r: Rect) -> FRect {
    FRect::new(r.x as f32, r.y as f32, r.w as f32, r.h as f32)
}

pub struct Gui<'a> {
    canvas: &'a mut WindowCanvas,
    textures: &'a TextureCreator<WindowContext>,
    pub(super) clipboard: &'a ClipboardUtil,
    pub input: Input,
    pub focus: Option<Field>,
    pub caret: Caret,
    pub changed: bool,
    pub(super) focus_by_tab: bool,
    interactive: bool,
    clips: Vec<Rect>,
}

pub fn clamp_scroll(offset: i32, delta: i32, content_h: i32, viewport_h: i32) -> i32 {
    (offset + delta).clamp(0, (content_h - viewport_h).max(0))
}

impl<'a> Gui<'a> {
    pub fn new(
        canvas: &'a mut WindowCanvas,
        textures: &'a TextureCreator<WindowContext>,
        clipboard: &'a ClipboardUtil,
        mut input: Input,
        focus: Option<Field>,
        mut caret: Caret,
    ) -> Self {
        let mut changed = false;
        let mut focus = focus;
        let mut focus_by_tab = false;
        if input.keys.contains(&Keycode::Tab) {
            input.keys.retain(|&k| k != Keycode::Tab);
            let next = focus.map(|f| f.tab_step(!input.shift));
            focus_by_tab = next != focus;
            focus = next;
            changed = true;
        }
        if !input.mouse_down {
            caret.dragging = false;
        }
        Self {
            canvas,
            textures,
            clipboard,
            input,
            focus,
            caret,
            changed,
            focus_by_tab,
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
        self.outline(rect, border);
    }

    pub fn outline(&mut self, rect: Rect, color: Color) {
        self.canvas.set_draw_color(color);
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

    pub fn scrollbar(&mut self, track: Rect, viewport_h: i32, offset: i32, content_h: i32) {
        if content_h <= viewport_h {
            return;
        }
        let thumb_h = (track.h * viewport_h / content_h).max(24);
        let travel = track.h - thumb_h;
        let max_offset = content_h - viewport_h;
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
}
