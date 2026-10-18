use sdl3::keyboard::Keycode;
use sdl3::ttf::Font;

use crate::focus::Field;

use super::charset::Charset;
use super::geometry::Rect;
use super::gui::Gui;
use super::text::{index_at, line_of, measure, word_span, wrap_spans};
use super::theme::{ACCENT_BORDER_SOFT, TEXT};

impl<'a> Gui<'a> {
    pub(super) fn edit_click(&mut self, field: Field, rect: Rect) -> Option<(i32, i32)> {
        let point = self.input.click?;
        if !self.take_click(rect) {
            return None;
        }
        self.focus = Some(field);
        Some(point)
    }

    pub(super) fn place_caret(&mut self, value: &str, i: usize, pressed: bool) {
        if !pressed {
            self.caret.select_to(i, true);
            return;
        }
        self.caret.dragging = true;
        match self.input.clicks {
            2 => {
                let (lo, hi) = word_span(value, i);
                self.caret.select_to(lo, false);
                self.caret.select_to(hi, true);
            }
            3.. => {
                self.caret.select_to(0, false);
                self.caret.select_to(value.len(), true);
            }
            _ => self.caret.select_to(i, self.input.shift),
        }
    }

    fn copy(&self, value: &str) {
        let (lo, hi) = self.caret.sel();
        if lo < hi {
            let _ = self.clipboard.set_clipboard_text(&value[lo..hi]);
        }
    }

    fn type_text(&mut self, value: &mut String, text: &str, multiline: bool, charset: Charset) -> bool {
        let accepted: String = text
            .chars()
            .filter(|&c| if c == '\n' { multiline } else { charset.accepts(c) })
            .collect();
        if accepted.is_empty() {
            return false;
        }
        let mut changed = self.caret.delete_sel(value);
        let room = charset.max_len().saturating_sub(value.chars().count());
        let clean: String = accepted.chars().take(room).collect();
        if !clean.is_empty() {
            self.caret.insert(value, &clean);
            changed = true;
        }
        changed
    }

    fn move_row(&mut self, font: &Font, value: &str, wrap_w: i32, down: bool) {
        let lines = wrap_spans(font, value, wrap_w);
        let row = line_of(&lines, self.caret.cursor);
        let target = if down {
            lines.get(row + 1)
        } else {
            row.checked_sub(1).and_then(|r| lines.get(r))
        };
        if let Some(&span) = target {
            let x = measure(font, &value[lines[row].0..self.caret.cursor]).0;
            let i = index_at(font, value, span, x);
            self.caret.select_to(i, self.input.shift);
        }
    }

    pub(super) fn apply_input(&mut self, font: &Font, value: &mut String, wrap_w: Option<i32>, charset: Charset) -> bool {
        let multiline = wrap_w.is_some();
        let (shift, ctrl) = (self.input.shift, self.input.ctrl);
        let text = std::mem::take(&mut self.input.text);
        let mut changed = self.type_text(value, &text, multiline, charset);
        for key in std::mem::take(&mut self.input.keys) {
            changed |= match key {
                Keycode::Backspace => self.caret.delete(value, true),
                Keycode::Delete => self.caret.delete(value, false),
                Keycode::Return | Keycode::KpEnter => self.type_text(value, "\n", multiline, charset),
                Keycode::Left | Keycode::Right => {
                    self.caret.nav(value, key == Keycode::Right, shift);
                    false
                }
                Keycode::Home | Keycode::End => {
                    let to = if key == Keycode::Home { 0 } else { value.len() };
                    self.caret.select_to(to, shift);
                    false
                }
                Keycode::Up | Keycode::Down => {
                    if let Some(w) = wrap_w {
                        self.move_row(font, value, w, key == Keycode::Down);
                    }
                    false
                }
                Keycode::A if ctrl => {
                    self.caret.select_to(0, false);
                    self.caret.select_to(value.len(), true);
                    false
                }
                Keycode::C if ctrl => {
                    self.copy(value);
                    false
                }
                Keycode::X if ctrl => {
                    self.copy(value);
                    self.caret.delete_sel(value)
                }
                Keycode::V if ctrl => match self.clipboard.clipboard_text() {
                    Ok(pasted) => self.type_text(value, &pasted.replace("\r\n", "\n"), multiline, charset),
                    Err(_) => false,
                },
                _ => false,
            };
        }
        if changed {
            self.changed = true;
        }
        changed
    }

    pub(super) fn follow_caret(&mut self, font: &Font, value: &str, view_w: i32) -> i32 {
        let text_w = measure(font, value).0;
        let cursor_x = measure(font, &value[..self.caret.cursor]).0;
        let mut scroll = self.caret.scroll.min((text_w - view_w + 2).max(0));
        if cursor_x - scroll > view_w - 2 {
            scroll = cursor_x - view_w + 2;
        }
        if cursor_x < scroll {
            scroll = cursor_x;
        }
        self.caret.scroll = scroll;
        scroll
    }

    pub(super) fn edit_line(&mut self, font: &Font, value: &str, (start, end): (usize, usize), x: i32, y: i32, lh: i32, live: bool) {
        if live {
            let (lo, hi) = self.caret.sel();
            let (lo, hi) = (lo.clamp(start, end), hi.clamp(start, end));
            if lo < hi {
                let x0 = x + measure(font, &value[start..lo]).0;
                let x1 = x + measure(font, &value[start..hi]).0;
                self.fill(Rect::new(x0, y, x1 - x0, lh), ACCENT_BORDER_SOFT);
            }
        }
        self.text(font, &value[start..end], x, y, TEXT);
        let cursor = self.caret.cursor;
        if live && self.input.caret_on && (start..=end).contains(&cursor) {
            self.fill(Rect::new(x + measure(font, &value[start..cursor]).0, y, 1, lh), TEXT);
        }
    }
}
