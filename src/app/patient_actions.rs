use super::{App, Page};
use crate::patient::Patient;

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
        self.show_leave_modal = false;
        self.new_session_notes.clear();
        self.new_session_objectives.clear();
        self.new_session_content.clear();
        self.editing_session = None;
        self.confirm_delete_session = None;
        self.confirm_delete = false;
        self.form_scroll = 0;
        self.sessions_scroll = 0;
        self.page = Page::PatientForm;
    }

    pub(super) fn delete_active(&mut self) {
        let id = self.active_patient.id.clone();
        self.patients.retain(|p| p.id != id);
        let _ = self.save(&self.patients);
        self.page = Page::Home;
    }

    pub(super) fn has_unsaved_changes(&self) -> bool {
        let mut current = self.active_patient.clone();
        current.sort_sessions();
        let saved = self
            .patients
            .iter()
            .find(|p| p.id == current.id)
            .cloned()
            .unwrap_or_else(|| Patient { id: current.id.clone(), ..Default::default() });
        current != saved
    }

    pub(super) fn save_active(&mut self) -> bool {
        let mut curr = self.active_patient.clone();
        curr.sort_sessions();
        curr.normalize();
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
        if self.save(&list).is_ok() {
            self.patients = list;
            self.page = Page::Home;
            true
        } else {
            self.validation_error = Some("Erreur lors de l'enregistrement sur le disque.".into());
            false
        }
    }
}
