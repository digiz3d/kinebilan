use gpui::{
    Context, IntoElement, Render, StatefulInteractiveElement as _, Window, div, prelude::*, rgb,
};

use crate::focus::Field;
use crate::patient::Patient;
use crate::{gpui_input::Input, gpui_ui};

gpui::actions!(gpui_access, [Submit, NextField, PreviousField]);

enum AccessMode {
    Setup,
    Unlock,
    Ready,
    LoadError(String),
}

fn password_error(password: &str, confirmation: &str) -> Option<&'static str> {
    if password.chars().count() < 8 {
        Some("Le mot de passe doit contenir au moins 8 caractères.")
    } else if password != confirmation {
        Some("Les deux mots de passe ne sont pas identiques.")
    } else {
        None
    }
}

fn search_key(text: &str) -> String {
    text.to_lowercase()
        .replace(['à', 'á', 'â', 'ã', 'ä', 'å'], "a")
        .replace('ç', "c")
        .replace(['è', 'é', 'ê', 'ë'], "e")
        .replace(['ì', 'í', 'î', 'ï'], "i")
        .replace(['ò', 'ó', 'ô', 'õ', 'ö'], "o")
        .replace(['ù', 'ú', 'û', 'ü'], "u")
        .replace(['ý', 'ÿ'], "y")
        .replace('œ', "oe")
        .replace('æ', "ae")
}

fn activity(patient: &Patient) -> String {
    patient
        .sessions
        .iter()
        .map(|s| s.date.as_str())
        .chain(std::iter::once(patient.creation_date.as_str()))
        .max()
        .unwrap_or("")
        .to_string()
}

fn tab_order(field: Field) -> isize {
    Field::tab_order_index(field).map_or(1, |index| index as isize + 1)
}

fn save_session(patient: &mut Patient, editing: Option<usize>, session: crate::patient::Session) {
    if let Some(index) = editing {
        if let Some(target) = patient.sessions.get_mut(index) {
            *target = session;
        }
    } else {
        patient.sessions.push(session);
    }
    patient.sort_sessions();
}

fn form_has_changes(
    original: &Patient,
    values: [&str; 8],
    sessions: &[crate::patient::Session],
) -> bool {
    values[0].trim() != original.last_name
        || values[1].trim() != original.first_name
        || values[2] != crate::patient::to_display_date(&original.birth_date)
        || values[3].trim() != original.care_type
        || values[4].trim() != original.antecedents
        || values[5].trim() != original.symptoms
        || values[6].trim() != original.treatment_objective
        || values[7] != crate::patient::to_display_datetime(&original.creation_date)
        || sessions != original.sessions
}

fn session_list_height(rows: &[gpui::Bounds<gpui::Pixels>]) -> Option<gpui::Pixels> {
    let first = rows.first()?;
    let second = rows.get(1).unwrap_or(first);
    Some(first.size.height + gpui::px(8.) + second.size.height * 0.75)
}

pub struct AccessView {
    mode: AccessMode,
    password: gpui::Entity<String>,
    confirmation: gpui::Entity<String>,
    _vault: Option<crate::storage::Vault>,
    patients: Vec<Patient>,
    search: gpui::Entity<String>,
    patients_scroll: gpui::ScrollHandle,
    form_scroll: gpui::ScrollHandle,
    sessions_scroll: gpui::ScrollHandle,
    sessions_height: gpui::Pixels,
    selected: Option<usize>,
    patient_open: bool,
    form: Vec<gpui::Entity<String>>,
    draft: Patient,
    session_date: gpui::Entity<String>,
    session_notes: gpui::Entity<String>,
    session_objectives: gpui::Entity<String>,
    session_content: gpui::Entity<String>,
    editing_session: Option<usize>,
    session_open: bool,
    pending_leave: bool,
    confirm_delete: bool,
    confirm_session_delete: Option<usize>,
    error: Option<String>,
}

impl AccessView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mode = match crate::storage::stored_kind() {
            Ok(crate::storage::Stored::Encrypted) => AccessMode::Unlock,
            Ok(crate::storage::Stored::Empty | crate::storage::Stored::Plaintext) => {
                AccessMode::Setup
            }
            Err(error) => AccessMode::LoadError(error.to_string()),
        };
        Self {
            mode,
            password: cx.new(|_| String::new()),
            confirmation: cx.new(|_| String::new()),
            _vault: None,
            patients: Vec::new(),
            search: cx.new(|_| String::new()),
            patients_scroll: gpui::ScrollHandle::new(),
            form_scroll: gpui::ScrollHandle::new(),
            sessions_scroll: gpui::ScrollHandle::new(),
            sessions_height: gpui::px(260.),
            selected: None,
            patient_open: false,
            form: (0..8).map(|_| cx.new(|_| String::new())).collect(),
            draft: Patient::default(),
            session_date: cx.new(|_| String::new()),
            session_notes: cx.new(|_| String::new()),
            session_objectives: cx.new(|_| String::new()),
            session_content: cx.new(|_| String::new()),
            editing_session: None,
            session_open: false,
            pending_leave: false,
            confirm_delete: false,
            confirm_session_delete: None,
            error: None,
        }
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let setup = matches!(self.mode, AccessMode::Setup);
        let password = self.password.read(cx).clone();
        let confirmation = self.confirmation.read(cx).clone();
        if setup && let Some(error) = password_error(&password, &confirmation) {
            self.error = Some(error.into());
        } else {
            let result = if setup {
                crate::storage::create_vault(&password)
            } else {
                crate::storage::unlock(&password)
            };
            match result {
                Ok((vault, patients)) => {
                    self._vault = Some(vault);
                    self.patients = patients;
                    self.password.update(cx, |value, cx| {
                        value.clear();
                        cx.notify();
                    });
                    self.confirmation.update(cx, |value, cx| {
                        value.clear();
                        cx.notify();
                    });
                    self.error = None;
                    self.mode = AccessMode::Ready;
                }
                Err(error) => self.error = Some(error.to_string()),
            }
        }
        cx.notify();
    }

    fn open_patient(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        self.selected = index;
        self.patient_open = true;
        self.error = None;
        self.pending_leave = false;
        self.confirm_delete = false;
        self.confirm_session_delete = None;
        self.editing_session = None;
        let patient = index
            .and_then(|i| self.patients.get(i))
            .cloned()
            .unwrap_or_else(|| Patient {
                id: uuid::Uuid::new_v4().to_string(),
                creation_date: crate::patient::format_stored_datetime(chrono::Utc::now()),
                ..Default::default()
            });
        self.draft = patient.clone();
        let values = [
            patient.last_name,
            patient.first_name,
            crate::patient::to_display_date(&patient.birth_date),
            patient.care_type,
            patient.antecedents,
            patient.symptoms,
            patient.treatment_objective,
            crate::patient::to_display_datetime(&patient.creation_date),
        ];
        for (field, value) in self.form.iter().zip(values) {
            field.update(cx, |stored, cx| {
                *stored = value;
                cx.notify();
            });
        }
        cx.notify();
    }

    fn save_patient(&mut self, leave: bool, cx: &mut Context<Self>) {
        let value = |index: usize| self.form[index].read(cx).trim().to_string();
        let (last_name, first_name, birth_display) = (value(0), value(1), value(2));
        let Some(birth_date) = crate::patient::to_storage_date(&birth_display) else {
            self.error =
                Some("Le nom, prénom et une date de naissance valide sont obligatoires.".into());
            cx.notify();
            return;
        };
        if last_name.is_empty() || first_name.is_empty() {
            self.error =
                Some("Le nom, prénom et une date de naissance valide sont obligatoires.".into());
            cx.notify();
            return;
        }
        let mut patient = self.draft.clone();
        if patient.id.is_empty() {
            patient.id = uuid::Uuid::new_v4().to_string();
        }
        patient.last_name = last_name;
        patient.first_name = first_name;
        patient.birth_date = birth_date;
        patient.care_type = value(3);
        patient.antecedents = value(4);
        patient.symptoms = value(5);
        patient.treatment_objective = value(6);
        let creation_display = value(7);
        if creation_display.is_empty() {
            patient.creation_date.clear();
        } else {
            let Some(created) = crate::patient::to_storage_datetime(&creation_display) else {
                self.error =
                    Some("La date de création doit être au format JJ/MM/AAAA HH:MM.".into());
                cx.notify();
                return;
            };
            patient.creation_date = created;
        }
        patient.normalize();
        let mut patients = self.patients.clone();
        let index = self.selected.unwrap_or(patients.len());
        if index < patients.len() {
            patients[index] = patient;
        } else {
            patients.push(patient);
        }
        match self
            ._vault
            .as_ref()
            .ok_or_else(|| std::io::Error::other("coffre non ouvert"))
            .and_then(|vault| crate::storage::save_patients(vault, &patients))
        {
            Ok(()) => {
                self.patients = patients;
                self.draft = self.patients[index].clone();
                self.selected = (!leave).then_some(index);
                self.patient_open = !leave;
                self.pending_leave = false;
                let saved = &self.draft;
                let values = [
                    saved.last_name.clone(),
                    saved.first_name.clone(),
                    crate::patient::to_display_date(&saved.birth_date),
                    saved.care_type.clone(),
                    saved.antecedents.clone(),
                    saved.symptoms.clone(),
                    saved.treatment_objective.clone(),
                    crate::patient::to_display_datetime(&saved.creation_date),
                ];
                for (field, value) in self.form.iter().zip(values) {
                    field.update(cx, |stored, cx| {
                        *stored = value;
                        cx.notify();
                    });
                }
                self.error = None;
            }
            Err(error) => self.error = Some(format!("Erreur lors de l'enregistrement : {error}")),
        }
        cx.notify();
    }

    fn add_session(&mut self, cx: &mut Context<Self>) {
        let date = self.session_date.read(cx).trim().to_string();
        let Some(date) = crate::patient::to_storage_date(&date) else {
            self.error = Some("La date de séance doit être au format JJ/MM/AAAA.".into());
            cx.notify();
            return;
        };
        let session = crate::patient::Session {
            id: self
                .editing_session
                .and_then(|i| self.draft.sessions.get(i))
                .map(|s| s.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            date,
            notes: self.session_notes.read(cx).trim().to_string(),
            objectives: self.session_objectives.read(cx).trim().to_string(),
            content: self.session_content.read(cx).trim().to_string(),
        };
        save_session(&mut self.draft, self.editing_session.take(), session);
        self.session_open = false;
        self.session_date.update(cx, |v, cx| {
            v.clear();
            cx.notify();
        });
        self.session_notes.update(cx, |v, cx| {
            v.clear();
            cx.notify();
        });
        self.session_objectives.update(cx, |v, cx| {
            v.clear();
            cx.notify();
        });
        self.session_content.update(cx, |v, cx| {
            v.clear();
            cx.notify();
        });
        self.error = None;
        cx.notify();
    }

    fn edit_session(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(session) = self.draft.sessions.get(index) {
            let values = [
                crate::patient::to_display_date(&session.date),
                session.notes.clone(),
                session.objectives.clone(),
                session.content.clone(),
            ];
            for (field, value) in [
                &self.session_date,
                &self.session_notes,
                &self.session_objectives,
                &self.session_content,
            ]
            .into_iter()
            .zip(values)
            {
                field.update(cx, |stored, cx| {
                    *stored = value;
                    cx.notify();
                });
            }
            self.editing_session = Some(index);
            self.session_open = true;
        }
        cx.notify();
    }

    fn new_session(&mut self, cx: &mut Context<Self>) {
        self.editing_session = None;
        self.session_open = true;
        self.session_date.update(cx, |v, cx| {
            *v = chrono::Local::now().format("%d/%m/%Y").to_string();
            cx.notify();
        });
        for field in [
            &self.session_notes,
            &self.session_objectives,
            &self.session_content,
        ] {
            field.update(cx, |v, cx| {
                v.clear();
                cx.notify();
            });
        }
        cx.notify();
    }

    fn has_unsaved_changes(&self, cx: &Context<Self>) -> bool {
        let new_patient = Patient {
            creation_date: self.draft.creation_date.clone(),
            ..Default::default()
        };
        let original = self
            .selected
            .and_then(|i| self.patients.get(i))
            .unwrap_or(&new_patient);
        form_has_changes(
            original,
            std::array::from_fn(|i| self.form[i].read(cx).as_str()),
            &self.draft.sessions,
        )
    }

    fn field(
        &self,
        id: &'static str,
        label: &'static str,
        value: gpui::Entity<String>,
        placeholder: &'static str,
    ) -> impl IntoElement {
        let input = Input::new(id, value)
            .placeholder(placeholder)
            .secret()
            .fill()
            .tab_order(tab_order(match id {
                "access-password" => Field::Password,
                _ => Field::PasswordConfirm,
            }));
        let input = if id == "access-password" {
            input.autofocus()
        } else {
            input
        };
        gpui_ui::form_field(label, input, None)
    }
}

impl Render for AccessView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let submit_on_enter = cx.listener(|this, _: &Submit, _, cx| this.submit(cx));
        let tab_forward = cx.listener(|_, _: &NextField, window, cx| window.focus_next(cx));
        let tab_backward = cx.listener(|_, _: &PreviousField, window, cx| window.focus_prev(cx));
        let mut leave_overlay = None;
        let card = match &self.mode {
            AccessMode::LoadError(message) => gpui_ui::card(
                "Impossible de charger les patients",
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(message.clone())
                    .child(gpui_ui::button(
                        "quit",
                        "Quitter",
                        gpui_ui::ButtonStyle::Primary,
                        |_, _, cx| cx.quit(),
                    )),
            ),
            AccessMode::Ready if self.patient_open => {
                let save = cx.listener(|this, _, _, cx| this.save_patient(false, cx));
                let save_and_leave = cx.listener(|this, _, _, cx| this.save_patient(true, cx));
                let back = cx.listener(|this, _, _, cx| {
                    if this.has_unsaved_changes(cx) {
                        this.pending_leave = true;
                    } else {
                        this.selected = None;
                        this.patient_open = false;
                        this.error = None;
                    }
                    cx.notify();
                });
                let leave = cx.listener(|this, _, _, cx| {
                    this.selected = None;
                    this.patient_open = false;
                    this.pending_leave = false;
                    this.error = None;
                    cx.notify();
                });
                let stay = cx.listener(|this, _, _, cx| {
                    this.pending_leave = false;
                    cx.notify();
                });
                let delete = cx.listener(|this, _, _, cx| {
                    if let Some(index) = this.selected {
                        if this.confirm_delete {
                            let mut patients = this.patients.clone();
                            patients.remove(index);
                            if let Some(vault) = &this._vault {
                                match crate::storage::save_patients(vault, &patients) {
                                    Ok(()) => {
                                        this.patients = patients;
                                        this.selected = None;
                                        this.patient_open = false;
                                        this.error = None;
                                        this.confirm_delete = false;
                                    }
                                    Err(error) => this.error = Some(error.to_string()),
                                }
                            }
                        } else {
                            this.confirm_delete = true;
                        }
                    }
                    cx.notify();
                });
                let text_field = |id: &'static str,
                                  label: &'static str,
                                  placeholder: &'static str,
                                  value: gpui::Entity<String>| {
                    let input = Input::new(id, value)
                        .placeholder(placeholder)
                        .fill()
                        .measure_width(gpui::px(160.))
                        .tab_order(tab_order(match id {
                            "patient-last" => Field::LastName,
                            "patient-first" => Field::FirstName,
                            "patient-birth" => Field::BirthDate,
                            "patient-care" => Field::CareType,
                            "patient-created" => Field::CreationDate,
                            "session-date" => Field::SessionDate,
                            _ => Field::Search,
                        }))
                        .focusable(!self.session_open && !self.pending_leave);
                    let input = match id {
                        "patient-birth" | "session-date" => input.date(),
                        "patient-created" => input.datetime(),
                        _ => input,
                    };
                    let input = if id == "patient-last" {
                        input.autofocus()
                    } else {
                        input
                    };
                    gpui_ui::form_field_bold(label, input)
                };
                let text_area = |id: &'static str,
                                 label: &'static str,
                                 placeholder: &'static str,
                                 value: gpui::Entity<String>| {
                    gpui_ui::form_field_bold(
                        label,
                        Input::new(id, value)
                            .placeholder(placeholder)
                            .fill()
                            .measure_width(gpui::px(820.))
                            .tab_order(tab_order(match id {
                                "patient-history" => Field::Antecedents,
                                "patient-symptoms" => Field::Symptoms,
                                "patient-objective" => Field::TreatmentObjective,
                                "session-notes" => Field::SessionNotes,
                                "session-objectives" => Field::SessionObjectives,
                                _ => Field::SessionContent,
                            }))
                            .multiline()
                            .focusable(!self.session_open && !self.pending_leave),
                    )
                };
                let new_session = cx.listener(|this, _, _, cx| this.new_session(cx));
                let measure_sessions =
                    cx.listener(|this, bounds: &Vec<gpui::Bounds<gpui::Pixels>>, _, cx| {
                        if let Some(height) = session_list_height(bounds)
                            && this.sessions_height != height
                        {
                            this.sessions_height = height;
                            cx.notify();
                        }
                    });
                let session_rows = self
                    .draft
                    .sessions
                    .iter()
                    .enumerate()
                    .map(|(index, session)| {
                        let remove = cx.listener(move |this, _, _, cx| {
                            if this.confirm_session_delete == Some(index) {
                                this.draft.sessions.remove(index);
                                this.confirm_session_delete = None;
                            } else {
                                this.confirm_session_delete = Some(index);
                            }
                            cx.notify();
                        });
                        let edit = cx.listener(move |this, _, _, cx| this.edit_session(index, cx));
                        let notes = if session.notes.trim().is_empty() {
                            "—".to_string()
                        } else {
                            session.notes.clone()
                        };
                        let objectives = if session.objectives.trim().is_empty() {
                            "—".to_string()
                        } else {
                            session.objectives.clone()
                        };
                        let content = if session.content.trim().is_empty() {
                            "—".to_string()
                        } else {
                            session.content.clone()
                        };
                        div()
                            .id(format!("session-row-{index}"))
                            .flex_shrink_0()
                            .flex()
                            .items_start()
                            .justify_between()
                            .gap_3()
                            .p_3()
                            .border_1()
                            .border_color(rgb(0xe5e7eb))
                            .bg(rgb(0xffffff))
                            .child(
                                div()
                                    .w(gpui::px(90.))
                                    .text_size(gpui::px(13.))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(0x1d4ed8))
                                    .child(crate::patient::to_display_date(&session.date)),
                            )
                            .child(
                                div()
                                    .flex_col()
                                    .flex_1()
                                    .child(
                                        div()
                                            .text_size(gpui::px(13.))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(0x374151))
                                            .child("Notes :"),
                                    )
                                    .child(
                                        div()
                                            .text_size(gpui::px(14.))
                                            .font_weight(gpui::FontWeight::NORMAL)
                                            .text_color(rgb(0x374151))
                                            .child(notes),
                                    )
                                    .child(
                                        div()
                                            .text_size(gpui::px(13.))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(0x374151))
                                            .child("Objectifs de la séance :"),
                                    )
                                    .child(
                                        div()
                                            .text_size(gpui::px(14.))
                                            .font_weight(gpui::FontWeight::NORMAL)
                                            .text_color(rgb(0x374151))
                                            .child(objectives),
                                    )
                                    .child(
                                        div()
                                            .text_size(gpui::px(13.))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(0x374151))
                                            .child("Contenu de la séance :"),
                                    )
                                    .child(
                                        div()
                                            .text_size(gpui::px(14.))
                                            .font_weight(gpui::FontWeight::NORMAL)
                                            .text_color(rgb(0x374151))
                                            .child(content),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(gpui_ui::sized_button(
                                        "session-delete",
                                        if self.confirm_session_delete == Some(index) {
                                            "Confirmer ?"
                                        } else {
                                            "Supprimer"
                                        },
                                        if self.confirm_session_delete == Some(index) {
                                            gpui_ui::ButtonStyle::DangerConfirm
                                        } else {
                                            gpui_ui::ButtonStyle::Danger
                                        },
                                        gpui::px(120.),
                                        gpui::px(30.),
                                        remove,
                                    ))
                                    .child(gpui_ui::sized_button(
                                        "session-edit",
                                        "Modifier",
                                        gpui_ui::ButtonStyle::Outline,
                                        gpui::px(96.),
                                        gpui::px(30.),
                                        edit,
                                    )),
                            )
                    })
                    .collect::<Vec<_>>();
                let form = div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(text_field("patient-last", "Nom *", "Nom de famille", self.form[0].clone()))
                            .child(text_field(
                                "patient-first",
                                "Prénom *",
                                "Prénom",
                                self.form[1].clone(),
                            ))
                            .child(text_field(
                                "patient-birth",
                                "Date de naissance *",
                                "JJ/MM/AAAA",
                                self.form[2].clone(),
                            ))
                            .child(text_field(
                                "patient-care",
                                "Prise en charge",
                                "Ex : Lombalgie",
                                self.form[3].clone(),
                            ))
                            .child(text_field(
                                "patient-created",
                                "Date de création",
                                "JJ/MM/AAAA HH:MM",
                                self.form[7].clone(),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_4()
                            .border_1()
                            .border_color(rgb(0xe5e7eb))
                            .bg(rgb(0xf9fafb))
                            .child(
                                div()
                                    .text_size(gpui::px(16.))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(0x111827))
                                    .child("Bilan clinique"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .child(text_area(
                                        "patient-history",
                                        "Antécédents",
                                        "Antécédents médicaux, chirurgicaux, traumatismes...",
                                        self.form[4].clone(),
                                    ))
                                    .child(text_area(
                                        "patient-symptoms",
                                        "Symptômes",
                                        "Douleurs, limitations articulaires, gênes fonctionnelles...",
                                        self.form[5].clone(),
                                    ))
                                    .child(text_area(
                                        "patient-objective",
                                        "Objectif de la prise en charge",
                                        "Objectif global du traitement de rééducation...",
                                        self.form[6].clone(),
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_4()
                            .border_1()
                            .border_color(rgb(0xe5e7eb))
                            .bg(rgb(0xf9fafb))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_size(gpui::px(16.))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(0x111827))
                                            .child("Sessions recap"),
                                    )
                                    .child(gpui_ui::sized_button(
                                        "session-new",
                                        "+ Nouvelle séance",
                                        gpui_ui::ButtonStyle::Outline,
                                        gpui::px(190.),
                                        gpui::px(36.),
                                        new_session,
                                    )),
                            )
                            .when(session_rows.is_empty(), |section| {
                                section.child(
                                    div()
                                        .text_color(rgb(0x9ca3af))
                                        .child("Aucune séance enregistrée."),
                                )
                            })
                            .when(!session_rows.is_empty(), |section| {
                                section.child(
                                    div()
                                        .h(self.sessions_height)
                                        .flex_shrink_0()
                                        .flex()
                                        .flex_col()
                                        .child(gpui_ui::scrollable(
                                            "sessions-recap-scroll",
                                            &self.sessions_scroll,
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_2()
                                                .children(session_rows)
                                                .on_children_prepainted(move |bounds, window, cx| {
                                                    measure_sessions(&bounds, window, cx);
                                                }),
                                        )),
                                )
                            }),
                    )
                    .when_some(self.error.clone(), |form, error| {
                        form.child(div().text_color(rgb(0xb91c1c)).child(error))
                    })
                    .child(div().flex().gap_2().child(gpui_ui::button(
                        "patient-delete",
                        if self.confirm_delete {
                            "Confirmer la suppression ?"
                        } else {
                            "Supprimer"
                        },
                        gpui_ui::ButtonStyle::Danger,
                        delete,
                    )));
                if self.pending_leave {
                    leave_overlay = Some(
                        div()
                            .w(gpui::px(480.))
                            .p_6()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(rgb(0xfca5a5))
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child("Modifications non enregistrées. Quitter sans enregistrer ?")
                            .when_some(self.error.clone(), |dialog, error| {
                                dialog.child(div().text_color(rgb(0xb91c1c)).child(error))
                            })
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(gpui_ui::button(
                                        "leave-stay",
                                        "Annuler",
                                        gpui_ui::ButtonStyle::Secondary,
                                        stay,
                                    ))
                                    .child(gpui_ui::button(
                                        "leave-discard",
                                        "Quitter sans enregistrer",
                                        gpui_ui::ButtonStyle::Danger,
                                        leave,
                                    ))
                                    .child(gpui_ui::button(
                                        "leave-save",
                                        "Enregistrer",
                                        gpui_ui::ButtonStyle::Primary,
                                        save_and_leave,
                                    )),
                            ),
                    );
                }
                gpui_ui::card(
                    "__full__",
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .h(gpui::px(64.))
                                .px_7()
                                .bg(rgb(0xf3f4f6))
                                .child(gpui_ui::sized_button(
                                    "patient-back-top",
                                    "← Retour",
                                    gpui_ui::ButtonStyle::Secondary,
                                    gpui::px(110.),
                                    gpui::px(40.),
                                    back,
                                ))
                                .child(
                                    div()
                                        .text_size(gpui::px(20.))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(rgb(0x111827))
                                        .child(
                                            if self
                                                .selected
                                                .is_some_and(|i| i < self.patients.len())
                                            {
                                                format!(
                                                    "Fiche patient : {} {}",
                                                    self.draft.last_name,
                                                    self.draft.display_first_name()
                                                )
                                            } else {
                                                "Nouveau patient".to_string()
                                            },
                                        ),
                                )
                                .child(gpui_ui::sized_button(
                                    "patient-save-top",
                                    "Enregistrer",
                                    gpui_ui::ButtonStyle::Primary,
                                    gpui::px(126.),
                                    gpui::px(40.),
                                    save,
                                )),
                        )
                        .child(gpui_ui::scrollable(
                            "patient-form-scroll",
                            &self.form_scroll,
                            div().p_7().child(form),
                        )),
                )
            }
            AccessMode::Ready => {
                let query = search_key(self.search.read(cx).trim());
                let mut matches = self
                    .patients
                    .iter()
                    .enumerate()
                    .filter(|(_, patient)| {
                        query.is_empty()
                            || [
                                patient.last_name.as_str(),
                                patient.first_name.as_str(),
                                patient.care_type.as_str(),
                            ]
                            .iter()
                            .any(|value| search_key(value).contains(&query))
                    })
                    .collect::<Vec<_>>();
                matches.sort_by_key(|(_, patient)| std::cmp::Reverse(activity(patient)));
                let no_matches = matches.is_empty();
                let rows = matches
                    .into_iter()
                    .map(|(index, patient)| {
                        let open =
                            cx.listener(move |this, _, _, cx| this.open_patient(Some(index), cx));
                        div()
                            .id(format!("patient-row-{index}"))
                            .flex()
                            .items_center()
                            .justify_between()
                            .h(gpui::px(70.))
                            .flex_shrink_0()
                            .px_4()
                            .border_1()
                            .border_color(rgb(0xe5e7eb))
                            .bg(rgb(0xfafafa))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0xeff6ff)))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_3()
                                            .child(
                                                div()
                                                    .text_size(gpui::px(16.))
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(rgb(0x1f2937))
                                                    .child(format!(
                                                        "{} {}",
                                                        patient.last_name.to_uppercase(),
                                                        patient.display_first_name()
                                                    )),
                                            )
                                            .when(!patient.care_type.is_empty(), |row| {
                                                row.child(
                                                    div()
                                                        .px_2()
                                                        .py_1()
                                                        .bg(rgb(0xe5e7eb))
                                                        .text_size(gpui::px(12.))
                                                        .font_weight(gpui::FontWeight::NORMAL)
                                                        .child(patient.care_type.clone()),
                                                )
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_size(gpui::px(13.))
                                            .font_weight(gpui::FontWeight::NORMAL)
                                            .text_color(rgb(0x6b7280))
                                            .child(format!(
                                                "Né(e) le: {}",
                                                crate::patient::to_display_date(
                                                    &patient.birth_date
                                                )
                                            )),
                                    ),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .text_size(gpui::px(12.))
                                    .font_weight(gpui::FontWeight::NORMAL)
                                    .bg(rgb(0xe0f2fe))
                                    .text_color(rgb(0x0369a1))
                                    .child(format!("{} séance(s)", patient.sessions.len())),
                            )
                            .on_click(open)
                    })
                    .collect::<Vec<_>>();
                let search = Input::new("patient-search", self.search.clone())
                    .placeholder("Rechercher par nom ou prénom...")
                    .fill()
                    .measure_width(gpui::px(920.))
                    .height(gpui::px(44.));
                let new_patient = cx.listener(|this, _, _, cx| this.open_patient(None, cx));
                gpui_ui::card(
                    "__full__",
                    div()
                        .flex()
                        .flex_col()
                        .h_full()
                        .flex_1()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .border_b_1()
                                .border_color(rgb(0xe5e7eb))
                                .flex_shrink_0()
                                .pb_4()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_size(gpui::px(26.))
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(rgb(0x111827))
                                                .child("KinéBilan"),
                                        )
                                        .child(
                                            div()
                                                .text_size(gpui::px(16.))
                                                .text_color(rgb(0x6b7280))
                                                .child("Dossiers des patients"),
                                        ),
                                )
                                .child(gpui_ui::sized_button(
                                    "patient-new",
                                    "Nouveau patient",
                                    gpui_ui::ButtonStyle::Primary,
                                    gpui::px(190.),
                                    gpui::px(40.),
                                    new_patient,
                                )),
                        )
                        .child(div().flex_shrink_0().child(search))
                        .when(no_matches, |content| content.child("Aucun patient trouvé."))
                        .child(gpui_ui::scrollable(
                            "patient-list-scroll",
                            &self.patients_scroll,
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .border_1()
                                .border_color(rgb(0xd1d5db))
                                .bg(rgb(0xffffff))
                                .children(rows),
                        )),
                )
            }
            AccessMode::Setup | AccessMode::Unlock => {
                let setup = matches!(self.mode, AccessMode::Setup);
                let (title, intro, action) = if setup {
                    (
                        "Protéger vos données",
                        "Vos données de patients seront chiffrées avec un mot de passe. Sans ce mot de passe, elles ne pourront pas être récupérées. Conservez-le en lieu sûr.",
                        "Créer",
                    )
                } else {
                    (
                        "Déverrouiller les données",
                        "Saisissez le mot de passe pour ouvrir les données des patients.",
                        "Déverrouiller",
                    )
                };
                let submit = cx.listener(|this, _, _, cx| this.submit(cx));
                gpui_ui::card(
                    "",
                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .p_6()
                        .border_1()
                        .border_color(rgb(0xe5e7eb))
                        .bg(rgb(0xffffff))
                        .child(
                            div()
                                .text_size(gpui::px(17.))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgb(0x111827))
                                .child(title),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .child(
                                    div()
                                        .text_size(gpui::px(14.))
                                        .line_height(gpui::px(17.))
                                        .font_weight(gpui::FontWeight::NORMAL)
                                        .text_color(rgb(0x1f2937))
                                        .child(intro),
                                )
                                .child(self.field(
                                    "access-password",
                                    "Mot de passe",
                                    self.password.clone(),
                                    if setup { "8 caractères minimum" } else { "" },
                                ))
                                .when(setup, |form| {
                                    form.child(self.field(
                                        "access-password-confirm",
                                        "Confirmation",
                                        self.confirmation.clone(),
                                        "",
                                    ))
                                })
                                .when_some(self.error.clone(), |form, error| {
                                    form.child(div().text_color(rgb(0xb91c1c)).child(error))
                                })
                                .child(
                                    div()
                                        .flex()
                                        .justify_end()
                                        .gap_3()
                                        .child(gpui_ui::sized_button(
                                            "access-submit",
                                            action,
                                            gpui_ui::ButtonStyle::Primary,
                                            gpui::px(160.),
                                            gpui::px(40.),
                                            submit,
                                        ))
                                        .child(gpui_ui::sized_button(
                                            "access-quit",
                                            "Quitter",
                                            gpui_ui::ButtonStyle::Secondary,
                                            gpui::px(110.),
                                            gpui::px(40.),
                                            |_, _, cx| cx.quit(),
                                        )),
                                ),
                        ),
                )
            }
        };

        let session_overlay = self.session_open.then(|| {
            let save = cx.listener(|this, _, _, cx| this.add_session(cx));
            let cancel = cx.listener(|this, _, _, cx| {
                this.session_open = false;
                this.editing_session = None;
                this.error = None;
                cx.notify();
            });
            let title = if self.editing_session.is_some() {
                "Modifier le compte-rendu de la séance"
            } else {
                "What did you do? (Compte-rendu de la séance)"
            };
            div().w(gpui::px(480.)).child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_6()
                    .border_1()
                    .border_color(rgb(0xe5e7eb))
                    .bg(rgb(0xffffff))
                    .child(
                        div()
                            .text_size(gpui::px(17.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child(title),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(gpui_ui::form_field_bold(
                                "Date de la séance",
                                Input::new("session-date", self.session_date.clone())
                                    .date()
                                    .placeholder("JJ/MM/AAAA")
                                    .measure_width(gpui::px(200.))
                                    .width(gpui::px(200.))
                                    .tab_order(tab_order(Field::SessionDate)),
                            ))
                            .when(
                                crate::patient::to_storage_date(self.session_date.read(cx).trim())
                                    .is_none(),
                                |body| {
                                    body.child(
                                        div().text_color(rgb(0xb91c1c)).child("Date invalide"),
                                    )
                                },
                            )
                            .child(gpui_ui::form_field_bold(
                                "Notes",
                                Input::new("session-notes", self.session_notes.clone())
                                    .placeholder("Remarques générales sur la séance...")
                                    .fill()
                                    .multiline()
                                    .tab_order(tab_order(Field::SessionNotes))
                                    .autofocus(),
                            ))
                            .child(gpui_ui::form_field_bold(
                                "Objectifs de la séance",
                                Input::new("session-objectives", self.session_objectives.clone())
                                    .placeholder("Objectifs de la séance et plan de rééducation...")
                                    .fill()
                                    .multiline()
                                    .tab_order(tab_order(Field::SessionObjectives)),
                            ))
                            .child(gpui_ui::form_field_bold(
                                "Contenu de la séance",
                                Input::new("session-content", self.session_content.clone())
                                    .placeholder("Qu'avez-vous fait pendant cette séance...")
                                    .fill()
                                    .multiline()
                                    .tab_order(tab_order(Field::SessionContent))
                                    .height(gpui::px(88.)),
                            ))
                            .when_some(self.error.clone(), |body, error| {
                                body.child(div().text_color(rgb(0xb91c1c)).child(error))
                            })
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap_3()
                                    .child(gpui_ui::sized_button(
                                        "session-cancel-modal",
                                        "Annuler",
                                        gpui_ui::ButtonStyle::Secondary,
                                        gpui::px(110.),
                                        gpui::px(40.),
                                        cancel,
                                    ))
                                    .child(gpui_ui::sized_button(
                                        "session-save-modal",
                                        if self.editing_session.is_some() {
                                            "Enregistrer"
                                        } else {
                                            "Enregistrer séance"
                                        },
                                        gpui_ui::ButtonStyle::Primary,
                                        gpui::px(170.),
                                        gpui::px(40.),
                                        save,
                                    )),
                            ),
                    ),
            )
        });

        div()
            .id("app-root")
            .relative()
            .when(matches!(self.mode, AccessMode::Ready), |root| {
                root.overflow_hidden()
            })
            .when(!matches!(self.mode, AccessMode::Ready), |root| {
                root.overflow_y_scroll()
            })
            .size_full()
            .bg(
                if matches!(
                    self.mode,
                    AccessMode::Setup | AccessMode::Unlock | AccessMode::LoadError(_)
                ) {
                    rgb(0x999999)
                } else {
                    rgb(0xffffff)
                },
            )
            .flex()
            .items_center()
            .justify_center()
            .font_family("Arial")
            .font_weight(gpui::FontWeight::NORMAL)
            .child(if self.patient_open {
                div().w_full().h_full().child(card)
            } else if matches!(self.mode, AccessMode::Ready) {
                div().w_full().h_full().p_7().child(card)
            } else {
                div()
                    .w(gpui::px(480.))
                    .max_w(gpui::relative(0.9))
                    .child(card)
            })
            .when_some(session_overlay.or(leave_overlay), |root, overlay| {
                root.child(
                    div()
                        .id("session-modal-backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(gpui::hsla(0., 0., 0.45, 0.4))
                        .on_click(|_, _, _| {})
                        .child(div().max_w(gpui::relative(0.9)).child(overlay)),
                )
            })
            .on_action(submit_on_enter)
            .on_action(tab_forward)
            .on_action(tab_backward)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        activity, form_has_changes, password_error, save_session, search_key, session_list_height,
        tab_order,
    };
    use crate::patient::{Patient, Session};

    #[test]
    fn recap_viewport_fits_one_row_and_three_quarters_of_the_next() {
        let row = |height| {
            gpui::Bounds::new(
                gpui::point(gpui::px(0.), gpui::px(0.)),
                gpui::size(gpui::px(600.), gpui::px(height)),
            )
        };
        assert_eq!(session_list_height(&[]), None);
        assert_eq!(session_list_height(&[row(100.)]), Some(gpui::px(183.)));
        assert_eq!(
            session_list_height(&[row(100.), row(200.), row(300.)]),
            Some(gpui::px(258.))
        );
    }

    #[test]
    fn new_patient_is_clean_until_fields_or_sessions_change() {
        let original = Patient {
            creation_date: "2026-11-08T10:00:00Z".into(),
            ..Default::default()
        };
        let created = crate::patient::to_display_datetime(&original.creation_date);
        let mut values = ["", "", "", "", "", "", "", created.as_str()];
        assert!(!form_has_changes(&original, values, &[]));
        values[0] = "Dupont";
        assert!(form_has_changes(&original, values, &[]));
        values[0] = "";
        assert!(form_has_changes(&original, values, &[Session::default()]));
        values[7] = "";
        assert!(form_has_changes(&original, values, &[]));
    }

    #[test]
    fn validates_setup_password_length_and_confirmation() {
        assert!(password_error("motdep", "motdep").is_some());
        assert!(password_error("motdepasse", "different").is_some());
        assert_eq!(password_error("motdepasse", "motdepasse"), None);
        assert_eq!(password_error("éééééééé", "éééééééé"), None);
    }

    #[test]
    fn search_is_accent_insensitive_and_activity_uses_latest_session() {
        assert_eq!(search_key("Élodie Œuvre"), "elodie oeuvre");
        let patient = Patient {
            creation_date: "2026-01-03T10:00:00Z".into(),
            sessions: vec![Session {
                date: "2026-10-20".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(activity(&patient), "2026-10-20");
    }

    #[test]
    fn tab_stops_follow_each_form_order_and_session_create_edit_is_saved() {
        use crate::focus::Field;
        for (field, expected) in [
            (Field::LastName, 1),
            (Field::FirstName, 2),
            (Field::BirthDate, 3),
            (Field::CareType, 4),
            (Field::CreationDate, 5),
            (Field::Antecedents, 6),
            (Field::Symptoms, 7),
            (Field::TreatmentObjective, 8),
            (Field::SessionDate, 1),
            (Field::SessionNotes, 2),
            (Field::SessionObjectives, 3),
            (Field::SessionContent, 4),
            (Field::Password, 1),
            (Field::PasswordConfirm, 2),
        ] {
            assert_eq!(tab_order(field), expected);
        }

        let mut patient = Patient::default();
        save_session(
            &mut patient,
            None,
            Session {
                id: "first".into(),
                date: "2026-01-02".into(),
                ..Default::default()
            },
        );
        save_session(
            &mut patient,
            Some(0),
            Session {
                id: "first".into(),
                date: "2026-01-03".into(),
                notes: "updated".into(),
                ..Default::default()
            },
        );
        assert_eq!(patient.sessions.len(), 1);
        assert_eq!(patient.sessions[0].date, "2026-01-03");
        assert_eq!(patient.sessions[0].notes, "updated");
    }
}
