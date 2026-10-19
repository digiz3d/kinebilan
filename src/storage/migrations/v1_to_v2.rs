use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Frozen copy of the version 1 shapes. Do not change these: they describe old files.
#[derive(Deserialize)]
struct FileV1 {
    patients: Vec<PatientV1>,
}

#[derive(Deserialize)]
struct PatientV1 {
    id: String,
    last_name: String,
    first_name: String,
    birth_date: String,
    #[serde(default)]
    care_type: String,
    #[serde(default, alias = "prior_issues")]
    antecedents: String,
    #[serde(default)]
    symptoms: String,
    #[serde(default)]
    treatment_objective: String,
    sessions: Vec<SessionV1>,
}

#[derive(Deserialize)]
struct SessionV1 {
    id: String,
    date: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    objectives: String,
    #[serde(default)]
    content: String,
}

/// Frozen copy of the version 2 shapes. Do not change these: they describe the version 2 output.
#[derive(Serialize)]
struct FileV2 {
    version: u32,
    patients: Vec<PatientV2>,
}

#[derive(Serialize)]
struct PatientV2 {
    id: String,
    last_name: String,
    first_name: String,
    birth_date: String,
    care_type: String,
    antecedents: String,
    symptoms: String,
    treatment_objective: String,
    sessions: Vec<SessionV2>,
}

#[derive(Serialize)]
struct SessionV2 {
    id: String,
    date: String,
    notes: String,
    objectives: String,
    content: String,
}

impl From<SessionV1> for SessionV2 {
    fn from(session: SessionV1) -> Self {
        SessionV2 {
            id: session.id,
            date: session.date,
            notes: session.notes,
            objectives: session.objectives,
            content: session.content,
        }
    }
}

impl From<PatientV1> for PatientV2 {
    fn from(patient: PatientV1) -> Self {
        PatientV2 {
            id: patient.id,
            last_name: patient.last_name,
            first_name: patient.first_name,
            birth_date: convert_birth_date(&patient.birth_date),
            care_type: patient.care_type,
            antecedents: patient.antecedents,
            symptoms: patient.symptoms,
            treatment_objective: patient.treatment_objective,
            sessions: patient.sessions.into_iter().map(SessionV2::from).collect(),
        }
    }
}

/// Converts a `ddmmyyyy`, `yyyy-mm-dd`, `dd-mm-yyyy` or `dd/mm/yyyy` birth date to `dd/mm/yyyy`.
/// Anything that is not a real date becomes an empty string.
fn convert_birth_date(raw: &str) -> String {
    let text = raw.trim();
    let date = if text.len() == 8 && text.bytes().all(|b| b.is_ascii_digit()) {
        let formatted = format!("{}/{}/{}", &text[..2], &text[2..4], &text[4..]);
        chrono::NaiveDate::parse_from_str(&formatted, "%d/%m/%Y").ok()
    } else {
        ["%Y-%m-%d", "%d-%m-%Y", "%d/%m/%Y"]
            .iter()
            .find_map(|format| chrono::NaiveDate::parse_from_str(text, format).ok())
    };
    date.map(|date| date.format("%d/%m/%Y").to_string()).unwrap_or_default()
}

/// Converts the patients' birth dates to the `dd/mm/yyyy` format.
pub fn migrate(raw: Value) -> Result<Value, serde_json::Error> {
    let old: FileV1 = serde_json::from_value(raw)?;
    let patients: Vec<PatientV2> = old.patients.into_iter().map(PatientV2::from).collect();
    serde_json::to_value(FileV2 { version: 2, patients })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations::test_support::assert_migration;
    use serde_json::json;

    #[test]
    fn test_converts_ddmmyyyy_birth_dates() {
        let raw = json!({
            "version": 1,
            "patients": [
                {
                    "id": "p-1",
                    "last_name": "Dupont",
                    "first_name": "Jean",
                    "birth_date": "12041985",
                    "prior_issues": "Lombalgie chronique",
                    "sessions": [
                        { "id": "s-1", "date": "2026-10-20", "notes": "Patient motivé" }
                    ]
                },
                {
                    "id": "p-2",
                    "last_name": "Martin",
                    "first_name": "Léa",
                    "birth_date": "31021990",
                    "sessions": []
                },
                {
                    "id": "p-3",
                    "last_name": "Durand",
                    "first_name": "Paul",
                    "birth_date": "1985-04-12",
                    "sessions": []
                },
                {
                    "id": "p-4",
                    "last_name": "Petit",
                    "first_name": "Marie",
                    "birth_date": " 01011990 ",
                    "sessions": []
                }
            ]
        });

        let expected = json!({
            "version": 2,
            "patients": [
                {
                    "id": "p-1",
                    "last_name": "Dupont",
                    "first_name": "Jean",
                    "birth_date": "12/04/1985",
                    "care_type": "",
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
                },
                {
                    "id": "p-3",
                    "last_name": "Durand",
                    "first_name": "Paul",
                    "birth_date": "12/04/1985",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                },
                {
                    "id": "p-4",
                    "last_name": "Petit",
                    "first_name": "Marie",
                    "birth_date": "01/01/1990",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                }
            ]
        });

        assert_migration(1, raw, expected);
    }

    #[test]
    fn test_converts_slashed_birth_dates() {
        let raw = json!({
            "version": 1,
            "patients": [
                { "id": "p-1", "last_name": "Dupont", "first_name": "Jean", "birth_date": "12/04/1985", "sessions": [] },
                { "id": "p-2", "last_name": "Durand", "first_name": "Paul", "birth_date": "31/02/1990", "sessions": [] }
            ]
        });

        let expected = json!({
            "version": 2,
            "patients": [
                {
                    "id": "p-1",
                    "last_name": "Dupont",
                    "first_name": "Jean",
                    "birth_date": "12/04/1985",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                },
                {
                    "id": "p-2",
                    "last_name": "Durand",
                    "first_name": "Paul",
                    "birth_date": "",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                }
            ]
        });

        assert_migration(1, raw, expected);
    }

    #[test]
    fn test_converts_dashed_birth_dates() {
        let raw = json!({
            "version": 1,
            "patients": [
                { "id": "p-1", "last_name": "Dupont", "first_name": "Jean", "birth_date": "1985-04-12", "sessions": [] },
                { "id": "p-2", "last_name": "Martin", "first_name": "Léa", "birth_date": "12-04-1985", "sessions": [] },
                { "id": "p-3", "last_name": "Durand", "first_name": "Paul", "birth_date": "1990-02-31", "sessions": [] },
                { "id": "p-4", "last_name": "Petit", "first_name": "Marie", "birth_date": "31-02-1990", "sessions": [] }
            ]
        });

        let expected = json!({
            "version": 2,
            "patients": [
                {
                    "id": "p-1",
                    "last_name": "Dupont",
                    "first_name": "Jean",
                    "birth_date": "12/04/1985",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                },
                {
                    "id": "p-2",
                    "last_name": "Martin",
                    "first_name": "Léa",
                    "birth_date": "12/04/1985",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                },
                {
                    "id": "p-3",
                    "last_name": "Durand",
                    "first_name": "Paul",
                    "birth_date": "",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                },
                {
                    "id": "p-4",
                    "last_name": "Petit",
                    "first_name": "Marie",
                    "birth_date": "",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                }
            ]
        });

        assert_migration(1, raw, expected);
    }

    #[test]
    fn test_rejects_non_object_input() {
        assert!(migrate(json!([])).is_err());
    }
}
