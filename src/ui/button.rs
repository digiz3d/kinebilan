use sdl3::pixels::Color;
use sdl3::ttf::Font;

use super::geometry::Rect;
use super::gui::Gui;
use super::theme::*;

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

impl<'a> Gui<'a> {
    pub fn button(&mut self, rect: Rect, label: &str, font: &Font, style: ButtonStyle) -> bool {
        let hot = self.hovered(rect);
        let (bg, fg, border) = style.colors(hot);
        self.fill_bordered(rect, bg, border);
        self.text_centered(font, label, rect, fg);
        self.take_click(rect)
    }
}
