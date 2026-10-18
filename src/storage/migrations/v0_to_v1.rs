use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::patient::{Patient, Session};

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

impl From<SessionV0> for Session {
    fn from(session: SessionV0) -> Self {
        Session {
            id: session.id,
            date: session.date,
            notes: session.notes,
            objectives: session.objectives,
            content: session.content,
        }
    }
}

impl From<PatientV0> for Patient {
    fn from(patient: PatientV0) -> Self {
        Patient {
            id: patient.id,
            last_name: patient.last_name,
            first_name: patient.first_name,
            birth_date: patient.birth_date,
            care_type: patient.care_type,
            antecedents: patient.antecedents,
            symptoms: patient.symptoms,
            treatment_objective: patient.treatment_objective,
            sessions: patient.sessions.into_iter().map(Session::from).collect(),
        }
    }
}

#[derive(Serialize)]
struct FileV1<'a> {
    version: u32,
    patients: &'a [Patient],
}

/// Wraps the bare patient array in the versioned `{ "version": 1, "patients": [...] }` shape.
pub fn migrate(raw: Value) -> Result<Value, serde_json::Error> {
    let old: Vec<PatientV0> = serde_json::from_value(raw)?;
    let patients: Vec<Patient> = old.into_iter().map(Patient::from).collect();
    serde_json::to_value(FileV1 { version: 1, patients: &patients })
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let migrated = migrate(raw).expect("migration should succeed");

        assert_eq!(migrated["version"], 1);
        let patients = migrated["patients"].as_array().expect("patients should be an array");
        assert_eq!(patients.len(), 2);
        assert_eq!(patients[0]["antecedents"], "Lombalgie chronique");
        assert_eq!(patients[0]["care_type"], "");
        assert_eq!(patients[0]["sessions"][0]["notes"], "Patient motivé");
        assert_eq!(patients[0]["sessions"][0]["objectives"], "");
        assert_eq!(patients[1]["sessions"], json!([]));
    }

    #[test]
    fn test_rejects_non_array_input() {
        assert!(migrate(json!({ "version": 0 })).is_err());
    }
}
