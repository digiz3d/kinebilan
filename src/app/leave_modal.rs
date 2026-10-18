use super::{App, FormAction};
use crate::ui::*;

impl App {
    pub(super) fn draw_leave_modal(&mut self, gui: &mut Gui, fonts: &Fonts, screen: Rect) -> FormAction {
        gui.fill(screen, BACKDROP);

        let pad = 24;
        let button_h = 40;
        let leave_label = "Quitter sans enregistrer";
        let leave_w = measure(&fonts.button, leave_label).0 + 40;
        let save_w = 150;
        let cancel_w = 110;
        let gap = 10;
        let buttons_w = cancel_w + gap + leave_w + gap + save_w;
        let dialog_w = (buttons_w + 2 * pad).max(480).min(screen.w * 9 / 10);
        let title_h = line_height(&fonts.modal_title);
        let body_h = line_height(&fonts.body);
        let dialog_h = pad + title_h + 16 + body_h + 4 + body_h + 24 + button_h + pad;
        let dialog = Rect::new(
            screen.x + (screen.w - dialog_w) / 2,
            screen.y + (screen.h - dialog_h) / 2,
            dialog_w,
            dialog_h,
        );
        gui.fill_bordered(dialog, WHITE, BORDER);

        let x = dialog.x + pad;
        let mut y = dialog.y + pad;
        gui.text(&fonts.modal_title, "Modifications non enregistrées", x, y, TITLE);
        y += title_h + 16;
        gui.text(&fonts.body, "Cette fiche contient des modifications qui ne sont pas enregistrées.", x, y, LABEL);
        y += body_h + 4;
        gui.text(&fonts.body, "Voulez-vous enregistrer avant de quitter ?", x, y, LABEL);
        y += body_h + 24;

        let save_rect = Rect::new(dialog.right() - pad - save_w, y, save_w, button_h);
        let leave_rect = Rect::new(save_rect.x - gap - leave_w, y, leave_w, button_h);
        let cancel_rect = Rect::new(leave_rect.x - gap - cancel_w, y, cancel_w, button_h);
        let mut action = FormAction::None;
        if gui.button(cancel_rect, "Annuler", &fonts.button, ButtonStyle::Secondary) {
            action = FormAction::CancelLeave;
        }
        if gui.button(leave_rect, leave_label, &fonts.button, ButtonStyle::Danger) {
            action = FormAction::LeaveWithoutSaving;
        }
        if gui.button(save_rect, "Enregistrer", &fonts.button, ButtonStyle::Primary) {
            action = FormAction::Save;
        }
        action
    }
}
