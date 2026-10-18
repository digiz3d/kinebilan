use super::{App, FormAction, Page, CARD_PADDING};
use crate::focus::Field;
use crate::patient::Patient;
use crate::storage::save_patients;
use crate::ui::*;

impl App {
    pub(super) fn start_new_patient(&mut self) {
        self.active_patient = Patient {
            id: uuid::Uuid::new_v4().to_string(),
            ..Default::default()
        };
        self.open_form();
    }

    pub(super) fn open_patient(&mut self, patient: Patient) {
        self.active_patient = patient;
        self.open_form();
    }

    fn open_form(&mut self) {
        self.validation_error = None;
        self.show_session_modal = false;
        self.new_session_notes.clear();
        self.confirm_delete = false;
        self.form_scroll = 0;
        self.sessions_scroll = 0;
        self.page = Page::PatientForm;
    }

    fn delete_active(&mut self) {
        let id = self.active_patient.id.clone();
        self.patients.retain(|p| p.id != id);
        let _ = save_patients(&self.patients);
        self.page = Page::Home;
    }

    fn save_active(&mut self) -> bool {
        let curr = self.active_patient.clone();
        if curr.last_name.trim().is_empty()
            || curr.first_name.trim().is_empty()
            || curr.birth_date.trim().is_empty()
        {
            self.validation_error =
                Some("Le nom, prénom et la date de naissance sont obligatoires.".into());
            return false;
        }
        let mut list = self.patients.clone();
        if let Some(idx) = list.iter().position(|p| p.id == curr.id) {
            list[idx] = curr;
        } else {
            list.push(curr);
        }
        if save_patients(&list).is_ok() {
            self.patients = list;
            self.page = Page::Home;
            true
        } else {
            self.validation_error = Some("Erreur lors de l'enregistrement sur le disque.".into());
            false
        }
    }

    pub(super) fn draw_form(&mut self, gui: &mut Gui, fonts: &Fonts, card: Rect, screen: Rect) {
        let viewport = card.inset(CARD_PADDING);
        let (x, w) = (viewport.x, viewport.w);
        let modal_open = self.show_session_modal;
        gui.set_interactive(!modal_open);
        let mut action = FormAction::None;

        gui.push_clip(card);
        let mut y = viewport.y - self.form_scroll;

        let header_h = 40;
        if gui.button(Rect::new(x, y, 110, header_h), "← Retour", &fonts.button, ButtonStyle::Secondary) {
            action = FormAction::Back;
        }
        let title = if self.active_patient.last_name.is_empty() {
            "Nouveau patient".to_string()
        } else {
            format!(
                "Fiche patient : {} {}",
                self.active_patient.last_name, self.active_patient.first_name
            )
        };
        gui.text_centered(&fonts.heading, &title, Rect::new(x + 120, y, w - 240, header_h), TITLE);
        y += header_h + 16;
        gui.fill(Rect::new(x, y, w, 1), DIVIDER);
        y += 1 + 20;

        if let Some(error) = self.validation_error.clone() {
            let alert = Rect::new(x, y, w, 44);
            gui.fill_bordered(alert, ALERT_BG, ALERT_BORDER);
            gui.text(&fonts.body, &error, x + 16, y + 12, ALERT_TEXT);
            y += 44 + 20;
        }

        let gap = 16;
        let col_w = (w - 2 * gap) / 3;
        let cols = [x, x + col_w + gap, x + 2 * (col_w + gap)];
        let label_h = line_height(&fonts.label);
        let field_h = 40;
        gui.text(&fonts.label, "Nom *", cols[0], y, LABEL);
        gui.text(&fonts.label, "Prénom *", cols[1], y, LABEL);
        gui.text(&fonts.label, "Date de naissance *", cols[2], y, LABEL);
        let field_y = y + label_h + 6;
        if gui.text_field(
            Field::LastName,
            Rect::new(cols[0], field_y, col_w, field_h),
            &mut self.active_patient.last_name,
            "Nom de famille",
            &fonts.body,
            Charset::Text,
        ) {
            self.validation_error = None;
        }
        if gui.text_field(
            Field::FirstName,
            Rect::new(cols[1], field_y, col_w, field_h),
            &mut self.active_patient.first_name,
            "Prénom",
            &fonts.body,
            Charset::Text,
        ) {
            self.validation_error = None;
        }
        if gui.text_field(
            Field::BirthDate,
            Rect::new(cols[2], field_y, col_w, field_h),
            &mut self.active_patient.birth_date,
            "AAAA-MM-JJ",
            &fonts.body,
            Charset::Date,
        ) {
            self.validation_error = None;
        }
        y = field_y + field_h + 20;

        y += self.draw_clinical(gui, fonts, x, y, w) + 20;

        let (sessions_h, sessions_action) = self.draw_sessions(gui, fonts, x, y, w);
        if sessions_action != FormAction::None {
            action = sessions_action;
        }
        y += sessions_h + 20;

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
        y = button_y + button_h;

        let content_bottom = y + self.form_scroll;
        let content_h = content_bottom - viewport.y;
        let delta = gui.take_scroll(card);
        self.form_scroll = clamp_scroll(self.form_scroll, delta, content_h, viewport.h);
        gui.scrollbar(Rect::new(card.right() - 6, viewport.y, 4, viewport.h), self.form_scroll, content_h);
        gui.pop_clip();

        gui.set_interactive(true);
        if modal_open {
            let modal_action = self.draw_session_modal(gui, fonts, screen);
            if modal_action != FormAction::None {
                action = modal_action;
            }
        }

        match action {
            FormAction::None => {}
            FormAction::Back => {
                self.page = Page::Home;
                gui.focus = None;
            }
            FormAction::Delete => {
                self.delete_active();
                gui.focus = None;
            }
            FormAction::Save => {
                if self.save_active() {
                    gui.focus = None;
                }
            }
            FormAction::OpenSessionModal => {
                self.new_session_notes.clear();
                self.show_session_modal = true;
                gui.focus = Some(Field::SessionNotes);
            }
            FormAction::CloseSessionModal => {
                self.show_session_modal = false;
                gui.focus = None;
            }
            FormAction::AddSession => {
                self.add_session();
                if !self.show_session_modal {
                    gui.focus = None;
                }
            }
        }
    }
}
