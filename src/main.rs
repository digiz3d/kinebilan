use std::path::PathBuf;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub id: String,
    pub date: String,
    pub notes: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Patient {
    pub id: String,
    pub last_name: String,
    pub first_name: String,
    pub birth_date: String,
    pub prior_issues: String,
    pub symptoms: String,
    pub goals: String,
    pub sessions: Vec<Session>,
}

#[derive(Clone, PartialEq)]
enum Page {
    Home,
    PatientForm,
}

fn storage_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        let dir = PathBuf::from(home).join(".kinebilan");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("patients.json")
    } else {
        PathBuf::from("patients.json")
    }
}

fn load_patients_from(path: &std::path::Path) -> Vec<Patient> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn save_patients_to(path: &std::path::Path, patients: &[Patient]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let data = serde_json::to_string_pretty(patients)?;
    std::fs::write(path, data)
}

fn load_patients() -> Vec<Patient> {
    load_patients_from(&storage_path())
}

fn save_patients(patients: &[Patient]) -> std::io::Result<()> {
    save_patients_to(&storage_path(), patients)
}

fn main() {
    let cfg = Config::new().with_window(
        WindowBuilder::new()
            .with_title("KinéBilan")
            .with_inner_size(dioxus::desktop::LogicalSize::new(980.0, 840.0)),
    );
    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

#[component]
fn App() -> Element {
    let mut patients = use_signal(load_patients);
    let mut current_page = use_signal(|| Page::Home);
    let search_query = use_signal(String::new);
    let mut active_patient = use_signal(Patient::default);
    let mut validation_error = use_signal(|| Option::<String>::None);
    let mut show_session_modal = use_signal(|| false);
    let mut new_session_notes = use_signal(String::new);

    let start_new_patient = move |_| {
        active_patient.set(Patient {
            id: uuid::Uuid::new_v4().to_string(),
            ..Default::default()
        });
        validation_error.set(None);
        show_session_modal.set(false);
        new_session_notes.set(String::new());
        current_page.set(Page::PatientForm);
    };

    let open_patient = move |patient: Patient| {
        active_patient.set(patient);
        validation_error.set(None);
        show_session_modal.set(false);
        new_session_notes.set(String::new());
        current_page.set(Page::PatientForm);
    };

    rsx! {
        style { {include_str!("style.css")} }
        div { class: "app-container",
            match *current_page.read() {
                Page::Home => rsx! {
                    HomeView {
                        patients,
                        search_query,
                        on_open: open_patient,
                        on_new: start_new_patient,
                    }
                },
                Page::PatientForm => rsx! {
                    PatientFormView {
                        patient: active_patient,
                        validation_error,
                        show_session_modal,
                        new_session_notes,
                        on_back: move |_| current_page.set(Page::Home),
                        on_delete: move |_| {
                            let curr_id = active_patient.read().id.clone();
                            let mut list = patients.read().clone();
                            list.retain(|p| p.id != curr_id);
                            let _ = save_patients(&list);
                            patients.set(list);
                            current_page.set(Page::Home);
                        },
                        on_save: move |_| {
                            let curr = active_patient.read().clone();
                            if curr.last_name.trim().is_empty()
                                || curr.first_name.trim().is_empty()
                                || curr.birth_date.trim().is_empty()
                            {
                                validation_error.set(Some("Le nom, prénom et la date de naissance sont obligatoires.".into()));
                                return;
                            }
                            let mut list = patients.read().clone();
                            if let Some(idx) = list.iter().position(|p| p.id == curr.id) {
                                list[idx] = curr;
                            } else {
                                list.push(curr);
                            }
                            if save_patients(&list).is_ok() {
                                patients.set(list);
                                current_page.set(Page::Home);
                            } else {
                                validation_error.set(Some("Erreur lors de l'enregistrement sur le disque.".into()));
                            }
                        },
                    }
                },
            }
        }
    }
}

#[component]
fn HomeView(
    patients: Signal<Vec<Patient>>,
    mut search_query: Signal<String>,
    on_open: EventHandler<Patient>,
    on_new: EventHandler<()>,
) -> Element {
    let query = search_query.read().to_lowercase();
    let patient_list = patients.read().clone();
    let filtered_patients: Vec<Patient> = patient_list
        .into_iter()
        .filter(|p| {
            if query.trim().is_empty() {
                true
            } else {
                p.last_name.to_lowercase().contains(&query)
                    || p.first_name.to_lowercase().contains(&query)
            }
        })
        .collect();

    rsx! {
        div { class: "page-card",
            div { class: "header-row",
                div {
                    h1 { "KinéBilan" }
                    p { class: "subtitle", "Dossiers des patients" }
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| on_new.call(()),
                    "+ Nouveau patient"
                }
            }

            div { class: "search-box",
                input {
                    r#type: "text",
                    placeholder: "Rechercher par nom ou prénom...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                }
            }

            div { class: "patient-list",
                if filtered_patients.is_empty() {
                    div { class: "empty-state",
                        p { "Aucun patient trouvé." }
                    }
                } else {
                    for p in filtered_patients {
                        div {
                            key: "{p.id}",
                            class: "patient-item",
                            onclick: {
                                let p_clone = p.clone();
                                move |_| on_open.call(p_clone.clone())
                            },
                            div { class: "patient-info",
                                span { class: "patient-name", "{p.last_name.to_uppercase()} {p.first_name}" }
                                span { class: "patient-dob", "Né(e) le: {p.birth_date}" }
                            }
                            span { class: "badge", "{p.sessions.len()} séance(s)" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PatientFormView(
    mut patient: Signal<Patient>,
    mut validation_error: Signal<Option<String>>,
    mut show_session_modal: Signal<bool>,
    mut new_session_notes: Signal<String>,
    on_back: EventHandler<()>,
    on_delete: EventHandler<()>,
    on_save: EventHandler<()>,
) -> Element {
    let mut confirm_delete = use_signal(|| false);
    let p = patient.read().clone();

    rsx! {
        div { class: "page-card",
            div { class: "header-row",
                button {
                    class: "btn btn-secondary",
                    onclick: move |_| on_back.call(()),
                    "← Retour"
                }
                h2 { if p.last_name.is_empty() { "Nouveau patient" } else { "Fiche patient : {p.last_name} {p.first_name}" } }
                div { style: "width: 80px;" }
            }

            if let Some(err) = validation_error.read().as_ref() {
                div { class: "alert-box", "{err}" }
            }

            div { class: "form-grid-3",
                div { class: "form-group",
                    label { "Nom *" }
                    input {
                        r#type: "text",
                        placeholder: "Nom de famille",
                        value: "{p.last_name}",
                        oninput: move |e| {
                            patient.write().last_name = e.value();
                            validation_error.set(None);
                        },
                    }
                }
                div { class: "form-group",
                    label { "Prénom *" }
                    input {
                        r#type: "text",
                        placeholder: "Prénom",
                        value: "{p.first_name}",
                        oninput: move |e| {
                            patient.write().first_name = e.value();
                            validation_error.set(None);
                        },
                    }
                }
                div { class: "form-group",
                    label { "Date de naissance *" }
                    input {
                        r#type: "date",
                        value: "{p.birth_date}",
                        oninput: move |e| {
                            patient.write().birth_date = e.value();
                            validation_error.set(None);
                        },
                    }
                }
            }

            div { class: "clinical-box",
                h3 { "Bilan clinique" }
                div { class: "form-group",
                    label { "Antécédents / Prior issues" }
                    textarea {
                        rows: "3",
                        placeholder: "Antécédents médicaux, chirurgicaux, traumatismes...",
                        value: "{p.prior_issues}",
                        oninput: move |e| patient.write().prior_issues = e.value(),
                    }
                }
                div { class: "form-group",
                    label { "Symptômes" }
                    textarea {
                        rows: "3",
                        placeholder: "Douleurs, limitations articulaires, gênes fonctionnelles...",
                        value: "{p.symptoms}",
                        oninput: move |e| patient.write().symptoms = e.value(),
                    }
                }
                div { class: "form-group",
                    label { "Objectifs thérapeutiques" }
                    textarea {
                        rows: "3",
                        placeholder: "Objectifs du patient et plan de rééducation...",
                        value: "{p.goals}",
                        oninput: move |e| patient.write().goals = e.value(),
                    }
                }
            }

            div { class: "sessions-box",
                div { class: "sessions-header",
                    h3 { "Sessions recap" }
                    button {
                        class: "btn btn-outline",
                        onclick: move |_| {
                            new_session_notes.set(String::new());
                            show_session_modal.set(true);
                        },
                        "+ Nouvelle séance"
                    }
                }

                div { class: "sessions-list",
                    if p.sessions.is_empty() {
                        p { class: "empty-text", "Aucune séance enregistrée." }
                    } else {
                        for s in p.sessions.iter().rev() {
                            div { key: "{s.id}", class: "session-item",
                                span { class: "session-date", "{s.date}" }
                                span { class: "session-notes", "\"{s.notes}\"" }
                            }
                        }
                    }
                }
            }

            div { class: "footer-row",
                button {
                    class: if *confirm_delete.read() { "btn btn-danger-confirm btn-large" } else { "btn btn-danger btn-large" },
                    onclick: move |_| {
                        if *confirm_delete.read() {
                            on_delete.call(());
                        } else {
                            confirm_delete.set(true);
                        }
                    },
                    if *confirm_delete.read() { "Confirmer la suppression ?" } else { "Supprimer le patient" }
                }
                button {
                    class: "btn btn-primary btn-large",
                    onclick: move |_| on_save.call(()),
                    "Enregistrer"
                }
            }

            if *show_session_modal.read() {
                div { class: "modal-backdrop",
                    div { class: "modal-dialog",
                        h3 { "What did you do? (Compte-rendu de la séance)" }
                        textarea {
                            rows: "4",
                            autofocus: true,
                            placeholder: "Qu'avez-vous fait pendant cette séance...",
                            value: "{new_session_notes}",
                            oninput: move |e| new_session_notes.set(e.value()),
                        }
                        div { class: "modal-actions",
                            button {
                                class: "btn btn-secondary",
                                onclick: move |_| show_session_modal.set(false),
                                "Annuler"
                            }
                            button {
                                class: "btn btn-primary",
                                onclick: move |_| {
                                    let notes = new_session_notes.read().trim().to_string();
                                    if !notes.is_empty() {
                                        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                                        patient.write().sessions.push(Session {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            date: today,
                                            notes,
                                        });
                                        new_session_notes.set(String::new());
                                        show_session_modal.set(false);
                                    }
                                },
                                "Enregistrer séance"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patient_validation() {
        let invalid1 = Patient {
            id: "1".into(),
            last_name: "".into(),
            first_name: "Jean".into(),
            birth_date: "1990-01-01".into(),
            ..Default::default()
        };
        let is_valid = |p: &Patient| {
            !p.last_name.trim().is_empty()
                && !p.first_name.trim().is_empty()
                && !p.birth_date.trim().is_empty()
        };
        assert!(!is_valid(&invalid1));

        let valid = Patient {
            id: "2".into(),
            last_name: "Dupont".into(),
            first_name: "Jean".into(),
            birth_date: "1990-01-01".into(),
            ..Default::default()
        };
        assert!(is_valid(&valid));
    }

    #[test]
    fn test_delete_patient_list() {
        let mut list = vec![
            Patient { id: "p-1".into(), ..Default::default() },
            Patient { id: "p-2".into(), ..Default::default() },
        ];
        list.retain(|p| p.id != "p-1");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "p-2");
    }

    #[test]
    fn test_save_and_load_patients() {
        let temp_dir = std::env::temp_dir().join(format!("kinebilan_test_{}", uuid::Uuid::new_v4()));
        let file_path = temp_dir.join("patients.json");

        let p1 = Patient {
            id: "p-1".into(),
            last_name: "Dupont".into(),
            first_name: "Jean".into(),
            birth_date: "1985-04-12".into(),
            prior_issues: "Lombalgie chronique".into(),
            symptoms: "Douleur L4-L5".into(),
            goals: "Reprise course à pied".into(),
            sessions: vec![Session {
                id: "s-1".into(),
                date: "2026-10-20".into(),
                notes: "Mobilisation lombaire et gainage".into(),
            }],
        };

        let list = vec![p1.clone()];
        save_patients_to(&file_path, &list).expect("Failed to save");

        let loaded = load_patients_from(&file_path);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0], p1);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
