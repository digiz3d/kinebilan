use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::patient::{Patient, Session};

/// Frozen copy of the version 2 shapes. Do not change these: they describe old files.
#[derive(Deserialize)]
struct FileV2 {
    patients: Vec<PatientV2>,
}

#[derive(Deserialize)]
struct PatientV2 {
    id: String,
    last_name: String,
    first_name: String,
    birth_date: String,
    #[serde(default)]
    care_type: String,
    #[serde(default)]
    antecedents: String,
    #[serde(default)]
    symptoms: String,
    #[serde(default)]
    treatment_objective: String,
    sessions: Vec<SessionV2>,
}

#[derive(Deserialize)]
struct SessionV2 {
    id: String,
    date: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    objectives: String,
    #[serde(default)]
    content: String,
}

#[derive(Serialize)]
struct FileV3<'a> {
    version: u32,
    patients: &'a [Patient],
}

impl From<SessionV2> for Session {
    fn from(session: SessionV2) -> Self {
        Session {
            id: session.id,
            date: session.date,
            notes: session.notes,
            objectives: session.objectives,
            content: session.content,
        }
    }
}

impl From<PatientV2> for Patient {
    fn from(patient: PatientV2) -> Self {
        Patient {
            id: patient.id,
            last_name: patient.last_name,
            first_name: patient.first_name,
            birth_date: patient.birth_date,
            creation_date: String::new(),
            care_type: patient.care_type,
            antecedents: patient.antecedents,
            symptoms: patient.symptoms,
            treatment_objective: patient.treatment_objective,
            sessions: patient.sessions.into_iter().map(Session::from).collect(),
        }
    }
}

/// Adds an empty creation date to each patient, since older files never recorded it.
pub fn migrate(raw: Value) -> Result<Value, serde_json::Error> {
    let old: FileV2 = serde_json::from_value(raw)?;
    let patients: Vec<Patient> = old.patients.into_iter().map(Patient::from).collect();
    serde_json::to_value(FileV3 { version: 3, patients: &patients })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations::test_support::assert_migration;
    use serde_json::json;

    #[test]
    fn test_adds_empty_creation_date() {
        let raw = json!({
            "version": 2,
            "patients": [
                {
                    "id": "p-1",
                    "last_name": "Dupont",
                    "first_name": "Jean",
                    "birth_date": "12/04/1985",
                    "care_type": "Lombalgie",
                    "antecedents": "Lombalgie chronique",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": [
                        { "id": "s-1", "date": "2026-10-20", "notes": "Patient motivé", "objectives": "", "content": "" }
                    ]
                },
                {
                    "id": "p-2",
                    "last_name": "Martin",
                    "first_name": "Léa",
                    "birth_date": "",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                }
            ]
        });

        let expected = json!({
            "version": 3,
            "patients": [
                {
                    "id": "p-1",
                    "last_name": "Dupont",
                    "first_name": "Jean",
                    "birth_date": "12/04/1985",
                    "creation_date": "",
                    "care_type": "Lombalgie",
                    "antecedents": "Lombalgie chronique",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": [
                        { "id": "s-1", "date": "2026-10-20", "notes": "Patient motivé", "objectives": "", "content": "" }
                    ]
                },
                {
                    "id": "p-2",
                    "last_name": "Martin",
                    "first_name": "Léa",
                    "birth_date": "",
                    "creation_date": "",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                }
            ]
        });

        assert_migration(2, raw, expected);
    }

    #[test]
    fn test_rejects_non_object_input() {
        assert!(migrate(json!([])).is_err());
    }
}
