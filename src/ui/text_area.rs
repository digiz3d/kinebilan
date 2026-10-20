use sdl3::mouse::SystemCursor;
use sdl3::ttf::Font;

use crate::focus::Field;

use super::charset::Charset;
use super::geometry::Rect;
use super::gui::Gui;
use super::text::{area_height, index_at, line_height, wrap_spans};
use super::theme::*;

impl<'a> Gui<'a> {
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
        let wrap_w = w - 2 * PAD_X;
        let click = self.edit_click(field, Rect::new(x, y, w, area_height(font, value, w, min_rows)));
        let focused = self.focus == Some(field);
        let lh = line_height(font);
        let text_x = x + PAD_X;
        let text_y = y + PAD_Y;
        if focused {
            self.caret.claim(field, value, self.focus_by_tab);
            self.caret.fit(value);
            let pointer = click.or(self.caret.dragging.then_some(self.input.mouse));
            if let Some((px, py)) = pointer {
                let lines = wrap_spans(font, value, wrap_w);
                let row = ((py - text_y) / lh).clamp(0, lines.len() as i32 - 1) as usize;
                let i = index_at(font, value, lines[row], px - text_x, false);
                self.place_caret(value, i, lines[row], click.is_some());
            }
            self.apply_input(font, value, Some(wrap_w), Charset::Text);
        }

        let h = area_height(font, value, w, min_rows);
        if self.hovered(Rect::new(x, y, w, h)) {
            self.cursor = SystemCursor::IBeam;
        }
        self.fill_bordered(Rect::new(x, y, w, h), WHITE, if focused { ACCENT } else { INPUT_BORDER });

        if value.is_empty() {
            self.text(font, placeholder, text_x, text_y, PLACEHOLDER);
        }
        for (i, line) in wrap_spans(font, value, wrap_w).into_iter().enumerate() {
            self.edit_line(font, value, line, text_x, text_y + i as i32 * lh, lh, focused, false);
        }

        h
    }
}
