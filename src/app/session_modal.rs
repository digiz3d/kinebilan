use super::sessions::is_valid_session_date;
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
        let notes_h = area_height(&fonts.body, &self.new_session_notes, inner_w, 3);
        let objectives_h = area_height(&fonts.body, &self.new_session_objectives, inner_w, 3);
        let content_h = area_height(&fonts.body, &self.new_session_content, inner_w, 4);
        let label_h = line_height(&fonts.label);
        let label_pitch = label_h + 6;
        let date_h = label_pitch + 40;
        let button_h = 40;
        let dialog_h = pad
            + title_h
            + 16
            + date_h
            + 16
            + label_pitch
            + notes_h
            + 16
            + label_pitch
            + objectives_h
            + 16
            + label_pitch
            + content_h
            + 16
            + button_h
            + pad;
        let dialog = Rect::new(
            screen.x + (screen.w - dialog_w) / 2,
            screen.y + (screen.h - dialog_h) / 2,
            dialog_w,
            dialog_h,
        );
        gui.fill_bordered(dialog, WHITE, BORDER);

        let x = dialog.x + pad;
        let mut y = dialog.y + pad;
        let (modal_title, save_label) = if self.editing_session.is_some() {
            ("Modifier le compte-rendu de la séance", "Enregistrer")
        } else {
            ("What did you do? (Compte-rendu de la séance)", "Enregistrer séance")
        };
        gui.text(&fonts.modal_title, modal_title, x, y, TITLE);
        y += title_h + 16;
        gui.text(&fonts.label, "Date de la séance", x, y, LABEL);
        let date_y = y + label_h + 6;
        let date_valid = is_valid_session_date(&self.new_session_date);
        gui.text_field(
            Field::SessionDate,
            Rect::new(x, date_y, 200, 40),
            &mut self.new_session_date,
            "AAAA-MM-JJ",
            &fonts.body,
            Charset::Date,
        );
        if !date_valid {
            gui.text(&fonts.label, "Date invalide", x + 212, date_y + 12, ALERT_TEXT);
        }
        y += date_h + 16;
        gui.text(&fonts.label, "Notes", x, y, LABEL);
        y += label_pitch;
        y += gui.text_area(
            Field::SessionNotes,
            x,
            y,
            inner_w,
            &mut self.new_session_notes,
            "Remarques générales sur la séance...",
            &fonts.body,
            3,
        ) + 16;
        gui.text(&fonts.label, "Objectifs de la séance", x, y, LABEL);
        y += label_pitch;
        y += gui.text_area(
            Field::SessionObjectives,
            x,
            y,
            inner_w,
            &mut self.new_session_objectives,
            "Objectifs de la séance et plan de rééducation...",
            &fonts.body,
            3,
        ) + 16;
        gui.text(&fonts.label, "Contenu de la séance", x, y, LABEL);
        y += label_pitch;
        gui.text_area(
            Field::SessionContent,
            x,
            y,
            inner_w,
            &mut self.new_session_content,
            "Qu'avez-vous fait pendant cette séance...",
            &fonts.body,
            4,
        );
        y += content_h + 16;

        let save_rect = Rect::new(dialog.right() - pad - 170, y, 170, button_h);
        let cancel_rect = Rect::new(save_rect.x - 10 - 110, y, 110, button_h);
        let mut action = FormAction::None;
        if gui.button(cancel_rect, "Annuler", &fonts.button, ButtonStyle::Secondary) {
            action = FormAction::CloseSessionModal;
        }
        if gui.button(save_rect, save_label, &fonts.button, ButtonStyle::Primary) {
            action = FormAction::SaveSession;
        }
        action
    }
}
