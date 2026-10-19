use super::{App, FormAction, CARD_PADDING};
use crate::ui::*;

pub(super) const BAR_H: i32 = 64;

impl App {
    /// Draws the static top bar with back, title and save. It does not scroll with the form.
    pub(super) fn draw_form_bar(&mut self, gui: &mut Gui, fonts: &Fonts, bar: Rect) -> FormAction {
        let mut action = FormAction::None;
        gui.fill(bar, BG);
        gui.fill(Rect::new(bar.x, bar.bottom() - 1, bar.w, 1), BORDER);

        let header_h = 40;
        let (x, y) = (bar.x + CARD_PADDING, bar.y + (bar.h - 1 - header_h) / 2);
        let w = bar.w - CARD_PADDING * 2;

        if gui.button(Rect::new(x, y, 110, header_h), "← Retour", &fonts.button, ButtonStyle::Secondary) {
            action = FormAction::Back;
        }
        let save_w = measure(&fonts.button, "Enregistrer").0 + 56;
        if gui.button(
            Rect::new(x + w - save_w, y, save_w, header_h),
            "Enregistrer",
            &fonts.button,
            ButtonStyle::Primary,
        ) {
            action = FormAction::Save;
        }

        let title = if self.active_patient.last_name.is_empty() {
            "Nouveau patient".to_string()
        } else {
            format!(
                "Fiche patient : {} {}",
                self.active_patient.last_name.to_uppercase(),
                self.active_patient.display_first_name()
            )
        };
        let side = 110.max(save_w) + 16;
        gui.text_centered(&fonts.heading, &title, Rect::new(x + side, y, w - side * 2, header_h), TITLE);
        action
    }
}
