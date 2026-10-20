use super::{App, Page};
use crate::patient::{format_stored_datetime, to_display_date, to_display_datetime, to_storage_date, to_storage_datetime, Patient};

impl App {
    pub(super) fn start_new_patient(&mut self) {
        self.active_patient = Patient {
            id: uuid::Uuid::new_v4().to_string(),
            creation_date: format_stored_datetime(chrono::Utc::now()),
            ..Default::default()
        };
        self.open_form();
    }

    pub(super) fn open_patient(&mut self, patient: Patient) {
        self.active_patient = patient;
        self.open_form();
    }

    /// Copies the stored dates of the active patient into the `dd/mm/yyyy` inputs.
    fn open_form(&mut self) {
        self.birth_date_input = to_display_date(&self.active_patient.birth_date);
        self.creation_date_input = to_display_datetime(&self.active_patient.creation_date);
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

    /// Stores the date inputs in the active patient. Input that is not a real date yet is kept as typed,
    /// so the patient still counts as changed and saving is refused until it is fixed.
    pub(super) fn sync_date_inputs(&mut self) {
        self.active_patient.birth_date =
            to_storage_date(&self.birth_date_input).unwrap_or_else(|| self.birth_date_input.clone());
        self.active_patient.creation_date =
            to_storage_datetime(&self.creation_date_input).unwrap_or_else(|| self.creation_date_input.clone());
    }

    pub(super) fn save_active(&mut self) -> bool {
        self.sync_date_inputs();
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
        if to_storage_date(&self.birth_date_input).is_none() {
            self.validation_error = Some("La date de naissance doit être au format JJ/MM/AAAA.".into());
            return false;
        }
        if !self.creation_date_input.trim().is_empty() && to_storage_datetime(&self.creation_date_input).is_none() {
            self.validation_error = Some("La date de création doit être au format JJ/MM/AAAA HH:MM.".into());
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
            true
        } else {
            self.validation_error = Some("Erreur lors de l'enregistrement sur le disque.".into());
            false
        }
    }
}
