use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::patient::{Patient, Session, to_storage_datetime};

/// Frozen copy of the version 3 shapes. Do not change these: they describe old files.
#[derive(Deserialize)]
struct FileV3 {
    patients: Vec<PatientV3>,
}

#[derive(Deserialize)]
struct PatientV3 {
    id: String,
    last_name: String,
    first_name: String,
    birth_date: String,
    creation_date: String,
    #[serde(default)]
    care_type: String,
    #[serde(default, alias = "prior_issues")]
    antecedents: String,
    #[serde(default)]
    symptoms: String,
    #[serde(default)]
    treatment_objective: String,
    sessions: Vec<SessionV3>,
}

#[derive(Deserialize)]
struct SessionV3 {
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
struct FileV4<'a> {
    version: u32,
    patients: &'a [Patient],
}

/// Converts a `dd/mm/yyyy` date to `yyyy-mm-dd`. Anything that is not a real date becomes an empty string.
fn convert_date(raw: &str) -> String {
    let text = raw.trim();
    if text.len() != 10 {
        return String::new();
    }
    chrono::NaiveDate::parse_from_str(text, "%d/%m/%Y")
        .map(|date| date.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// Turns a `dd/mm/yyyy` creation date into local midnight, stored in UTC. Anything else becomes an empty string.
fn convert_creation_date(raw: &str) -> String {
    let text = raw.trim();
    if text.len() != 10 {
        return String::new();
    }
    to_storage_datetime(&format!("{text} 00:00")).unwrap_or_default()
}

impl From<SessionV3> for Session {
    fn from(session: SessionV3) -> Self {
        Session {
            id: session.id,
            date: session.date,
            notes: session.notes,
            objectives: session.objectives,
            content: session.content,
        }
    }
}

impl From<PatientV3> for Patient {
    fn from(patient: PatientV3) -> Self {
        Patient {
            id: patient.id,
            last_name: patient.last_name,
            first_name: patient.first_name,
            birth_date: convert_date(&patient.birth_date),
            creation_date: convert_creation_date(&patient.creation_date),
            care_type: patient.care_type,
            antecedents: patient.antecedents,
            symptoms: patient.symptoms,
            treatment_objective: patient.treatment_objective,
            sessions: patient.sessions.into_iter().map(Session::from).collect(),
        }
    }
}

/// Stores birth dates as `yyyy-mm-dd` and creation dates as UTC date and time. Session dates were already `yyyy-mm-dd`.
/// Creation dates only have a day, so they become local midnight.
pub fn migrate(raw: Value) -> Result<Value, serde_json::Error> {
    let old: FileV3 = serde_json::from_value(raw)?;
    let patients: Vec<Patient> = old.patients.into_iter().map(Patient::from).collect();
    serde_json::to_value(FileV4 { version: 4, patients: &patients })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations::test_support::assert_migration;
    use serde_json::json;

    #[test]
    fn test_converts_dates_to_iso() {
        let raw = json!({
            "version": 3,
            "patients": [
                {
                    "id": "p-1",
                    "last_name": "Dupont",
                    "first_name": "Jean",
                    "birth_date": "12/04/1985",
                    "creation_date": "20/10/2026",
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
                    "birth_date": "31/02/1990",
                    "creation_date": "",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                }
            ]
        });

        let midnight = to_storage_datetime("20/10/2026 00:00").unwrap();
        let expected = json!({
            "version": 4,
            "patients": [
                {
                    "id": "p-1",
                    "last_name": "Dupont",
                    "first_name": "Jean",
                    "birth_date": "1985-04-12",
                    "creation_date": midnight,
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

        assert_migration(3, raw, expected);
    }

    #[test]
    fn test_rejects_non_object_input() {
        assert!(migrate(json!([])).is_err());
    }
}
