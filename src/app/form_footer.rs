use super::{App, FormAction};
use crate::ui::*;

impl App {
    /// Draws the delete and save buttons at y and returns the requested action and the bottom edge.
    pub(super) fn draw_form_footer(&mut self, gui: &mut Gui, fonts: &Fonts, x: i32, y: i32, w: i32) -> (FormAction, i32) {
        let mut action = FormAction::None;
        gui.fill(Rect::new(x, y, w, 1), DIVIDER);
        let button_y = y + 12;
        let button_h = 46;
        let delete_label = if self.confirm_delete {
            "Confirmer la suppression ?"
        } else {
            "Supprimer le patient"
        };
        let delete_w = measure(&fonts.button_large, "Confirmer la suppression ?").0
            .max(measure(&fonts.button_large, "Supprimer le patient").0)
            + 56;
        let delete_style = if self.confirm_delete {
            ButtonStyle::DangerConfirm
        } else {
            ButtonStyle::Danger
        };
        if gui.button(
            Rect::new(x, button_y, delete_w, button_h),
            delete_label,
            &fonts.button_large,
            delete_style,
        ) {
            if self.confirm_delete {
                action = FormAction::Delete;
            } else {
                self.confirm_delete = true;
            }
        }
        let save_w = measure(&fonts.button_large, "Enregistrer").0 + 56;
        if gui.button(
            Rect::new(x + w - save_w, button_y, save_w, button_h),
            "Enregistrer",
            &fonts.button_large,
            ButtonStyle::Primary,
        ) {
            action = FormAction::Save;
        }
        (action, button_y + button_h)
    }
}
