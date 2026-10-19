use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Frozen copy of the version 0 shapes. Do not change these: they describe old files.
#[derive(Deserialize)]
struct PatientV0 {
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
    sessions: Vec<SessionV0>,
}

#[derive(Deserialize)]
struct SessionV0 {
    id: String,
    date: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    objectives: String,
    #[serde(default)]
    content: String,
}

/// Frozen copy of the version 1 shapes, as written by this step. Do not change these.
#[derive(Serialize)]
struct FileV1 {
    version: u32,
    patients: Vec<PatientV1>,
}

#[derive(Serialize)]
struct PatientV1 {
    id: String,
    last_name: String,
    first_name: String,
    birth_date: String,
    care_type: String,
    antecedents: String,
    symptoms: String,
    treatment_objective: String,
    sessions: Vec<SessionV1>,
}

#[derive(Serialize)]
struct SessionV1 {
    id: String,
    date: String,
    notes: String,
    objectives: String,
    content: String,
}

impl From<SessionV0> for SessionV1 {
    fn from(session: SessionV0) -> Self {
        SessionV1 {
            id: session.id,
            date: session.date,
            notes: session.notes,
            objectives: session.objectives,
            content: session.content,
        }
    }
}

impl From<PatientV0> for PatientV1 {
    fn from(patient: PatientV0) -> Self {
        PatientV1 {
            id: patient.id,
            last_name: patient.last_name,
            first_name: patient.first_name,
            birth_date: patient.birth_date,
            care_type: patient.care_type,
            antecedents: patient.antecedents,
            symptoms: patient.symptoms,
            treatment_objective: patient.treatment_objective,
            sessions: patient.sessions.into_iter().map(SessionV1::from).collect(),
        }
    }
}

/// Wraps the bare patient array in the versioned `{ "version": 1, "patients": [...] }` shape.
pub fn migrate(raw: Value) -> Result<Value, serde_json::Error> {
    let old: Vec<PatientV0> = serde_json::from_value(raw)?;
    let patients: Vec<PatientV1> = old.into_iter().map(PatientV1::from).collect();
    serde_json::to_value(FileV1 { version: 1, patients })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations::test_support::assert_migration;
    use serde_json::json;

    #[test]
    fn test_wraps_bare_array_with_version() {
        let raw = json!([
            {
                "id": "p-1",
                "last_name": "Dupont",
                "first_name": "Jean",
                "birth_date": "1985-04-12",
                "prior_issues": "Lombalgie chronique",
                "sessions": [
                    { "id": "s-1", "date": "2026-10-20", "notes": "Patient motivé" }
                ]
            },
            {
                "id": "p-2",
                "last_name": "Martin",
                "first_name": "Léa",
                "birth_date": "1990-01-01",
                "sessions": []
            }
        ]);

        let expected = json!({
            "version": 1,
            "patients": [
                {
                    "id": "p-1",
                    "last_name": "Dupont",
                    "first_name": "Jean",
                    "birth_date": "1985-04-12",
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
                    "birth_date": "1990-01-01",
                    "care_type": "",
                    "antecedents": "",
                    "symptoms": "",
                    "treatment_objective": "",
                    "sessions": []
                }
            ]
        });

        assert_migration(0, raw, expected);
    }

    #[test]
    fn test_rejects_non_array_input() {
        assert!(migrate(json!({ "version": 0 })).is_err());
    }
}
