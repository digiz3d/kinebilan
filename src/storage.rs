use std::path::PathBuf;

use crate::patient::Patient;

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

pub fn load_patients() -> Vec<Patient> {
    load_patients_from(&storage_path())
}

pub fn save_patients(patients: &[Patient]) -> std::io::Result<()> {
    save_patients_to(&storage_path(), patients)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patient::Session;

    #[test]
    fn test_save_and_load_patients() {
        let temp_dir = std::env::temp_dir().join(format!("kinebilan_test_{}", uuid::Uuid::new_v4()));
        let file_path = temp_dir.join("patients.json");

        let p1 = Patient {
            id: "p-1".into(),
            last_name: "Dupont".into(),
            first_name: "Jean".into(),
            birth_date: "1985-04-12".into(),
            care_type: "Lombalgie".into(),
            antecedents: "Lombalgie chronique".into(),
            symptoms: "Douleur L4-L5".into(),
            treatment_objective: "Reprise course à pied".into(),
            sessions: vec![Session {
                id: "s-1".into(),
                date: "2026-10-20".into(),
                notes: "Patient motivé".into(),
                objectives: "Mobilité lombaire".into(),
                content: "Mobilisation lombaire et gainage".into(),
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
