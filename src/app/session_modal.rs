use super::{App, FormAction};
use crate::focus::Field;
use crate::ui::*;

impl App {
    pub(super) fn draw_session_modal(&mut self, gui: &mut Gui, fonts: &Fonts, screen: Rect) -> FormAction {
        gui.fill(screen, BACKDROP);

        let pad = 24;
        let dialog_w = 480.min(screen.w * 9 / 10);
        let inner_w = dialog_w - 2 * pad;
        let title_h = line_height(&fonts.modal_title);
        let notes_h = area_height(&fonts.body, &self.new_session_notes, inner_w, 4);
        let button_h = 40;
        let dialog_h = pad + title_h + 16 + notes_h + 16 + button_h + pad;
        let dialog = Rect::new(
            screen.x + (screen.w - dialog_w) / 2,
            screen.y + (screen.h - dialog_h) / 2,
            dialog_w,
            dialog_h,
        );
        gui.fill_bordered(dialog, WHITE, BORDER);

        let x = dialog.x + pad;
        let mut y = dialog.y + pad;
        gui.text(&fonts.modal_title, "What did you do? (Compte-rendu de la séance)", x, y, TITLE);
        y += title_h + 16;
        gui.text_area(
            Field::SessionNotes,
            x,
            y,
            inner_w,
            &mut self.new_session_notes,
            "Qu'avez-vous fait pendant cette séance...",
            &fonts.body,
            4,
        );
        y += notes_h + 16;

        let save_rect = Rect::new(dialog.right() - pad - 170, y, 170, button_h);
        let cancel_rect = Rect::new(save_rect.x - 10 - 110, y, 110, button_h);
        let mut action = FormAction::None;
        if gui.button(cancel_rect, "Annuler", &fonts.button, ButtonStyle::Secondary) {
            action = FormAction::CloseSessionModal;
        }
        if gui.button(save_rect, "Enregistrer séance", &fonts.button, ButtonStyle::Primary) {
            action = FormAction::AddSession;
        }
        action
    }
}
