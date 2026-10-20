use sdl3::mouse::SystemCursor;
use sdl3::ttf::Font;

use crate::focus::Field;

use super::charset::Charset;
use super::geometry::Rect;
use super::gui::Gui;
use super::text::{index_at, line_height};
use super::theme::*;

impl<'a> Gui<'a> {
    pub fn text_field(
        &mut self,
        field: Field,
        rect: Rect,
        value: &mut String,
        placeholder: &str,
        font: &Font,
        charset: Charset,
    ) -> bool {
        let click = self.edit_click(field, rect);
        let focused = self.focus == Some(field);
        let secret = charset == Charset::Secret;
        let inner = Rect::new(rect.x + PAD_X, rect.y, rect.w - 2 * PAD_X, rect.h);
        let mut changed = false;
        let mut scroll = 0;
        if focused {
            self.caret.claim(field, value, self.focus_by_tab);
            self.caret.fit(value);
            let pointer = click.or(self.caret.dragging.then_some(self.input.mouse));
            if let Some((px, _)) = pointer {
                let i = index_at(font, value, (0, value.len()), px - inner.x + self.caret.scroll, secret);
                self.place_caret(value, i, (0, value.len()), click.is_some());
            }
            changed = self.apply_input(font, value, None, charset);
            scroll = self.follow_caret(font, value, inner.w, secret);
        }

        if self.hovered(rect) {
            self.cursor = SystemCursor::IBeam;
        }
        let lh = line_height(font);
        let text_y = rect.y + (rect.h - lh) / 2;
        self.fill_bordered(rect, WHITE, if focused { ACCENT } else { INPUT_BORDER });

        self.push_clip(inner);
        if value.is_empty() {
            self.text(font, placeholder, inner.x, text_y, PLACEHOLDER);
        }
        self.edit_line(font, value, (0, value.len()), inner.x - scroll, text_y, lh, focused, secret);
        self.pop_clip();

        changed
    }
}
