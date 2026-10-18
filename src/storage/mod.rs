mod migrations;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::patient::Patient;

pub const CURRENT_VERSION: u32 = 1;

#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    Invalid(String),
    NewerVersion { found: u32, supported: u32 },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Impossible de lire le fichier de sauvegarde : {error}"),
            Self::Invalid(reason) => write!(f, "Le fichier de sauvegarde est illisible : {reason}"),
            Self::NewerVersion { found, supported } => write!(
                f,
                "Le fichier de sauvegarde est au format version {found}, mais cette version de KinéBilan ne gère que la version {supported}. Mettez l'application à jour pour l'ouvrir."
            ),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    patients: Vec<Patient>,
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

/// A bare JSON array is the unversioned format, which is version 0.
fn detect_version(raw: &Value) -> Result<u32, LoadError> {
    match raw {
        Value::Array(_) => Ok(0),
        Value::Object(map) => map
            .get("version")
            .and_then(Value::as_u64)
            .and_then(|version| u32::try_from(version).ok())
            .ok_or_else(|| LoadError::Invalid("champ \"version\" absent ou invalide".into())),
        _ => Err(LoadError::Invalid("format inconnu".into())),
    }
}

fn migrate(mut raw: Value, from: u32) -> Result<Vec<Patient>, LoadError> {
    for version in from..CURRENT_VERSION {
        let step = migrations::step(version).ok_or_else(|| {
            LoadError::Invalid(format!("aucune migration depuis la version {version}"))
        })?;
        raw = step(raw).map_err(|error| LoadError::Invalid(error.to_string()))?;
    }
    let file: SaveFile =
        serde_json::from_value(raw).map_err(|error| LoadError::Invalid(error.to_string()))?;
    Ok(file.patients)
}

fn load_patients_from(path: &Path) -> Result<Vec<Patient>, LoadError> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(LoadError::Io(error)),
    };
    let raw: Value =
        serde_json::from_str(&content).map_err(|error| LoadError::Invalid(error.to_string()))?;
    let version = detect_version(&raw)?;

    if version > CURRENT_VERSION {
        return Err(LoadError::NewerVersion { found: version, supported: CURRENT_VERSION });
    }
    if version == CURRENT_VERSION {
        return migrate(raw, version);
    }

    let backup = path.with_extension(format!("json.bak-v{version}"));
    std::fs::copy(path, &backup).map_err(LoadError::Io)?;
    let patients = migrate(raw, version)?;
    save_patients_to(path, &patients).map_err(LoadError::Io)?;
    Ok(patients)
}

fn save_patients_to(path: &Path, patients: &[Patient]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = SaveFile { version: CURRENT_VERSION, patients: patients.to_vec() };
    let data = serde_json::to_string_pretty(&file)?;
    std::fs::write(path, data)
}

pub fn load_patients() -> Result<Vec<Patient>, LoadError> {
    load_patients_from(&storage_path())
}

pub fn save_patients(patients: &[Patient]) -> std::io::Result<()> {
    save_patients_to(&storage_path(), patients)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patient::Session;

    const V0_FILE: &str = r#"[
  {
    "id": "p-1",
    "last_name": "Dupont",
    "first_name": "Jean",
    "birth_date": "1985-04-12",
    "prior_issues": "Lombalgie chronique",
    "sessions": [
      { "id": "s-1", "date": "2026-10-20", "notes": "Patient motivé" }
    ]
  }
]"#;

    fn temp_patients_file() -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("kinebilan_test_{}", uuid::Uuid::new_v4()));
        (dir.clone(), dir.join("patients.json"))
    }

    fn sample_patient() -> Patient {
        Patient {
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
        }
    }

    #[test]
    fn test_save_and_load_patients() {
        let (temp_dir, file_path) = temp_patients_file();
        let p1 = sample_patient();

        save_patients_to(&file_path, std::slice::from_ref(&p1)).expect("Failed to save");

        let loaded = load_patients_from(&file_path).expect("Failed to load");
        assert_eq!(loaded, vec![p1]);

        let raw: Value = serde_json::from_str(&std::fs::read_to_string(&file_path).unwrap()).unwrap();
        assert_eq!(raw["version"], CURRENT_VERSION);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_missing_file_loads_empty() {
        let (temp_dir, file_path) = temp_patients_file();
        assert!(load_patients_from(&file_path).unwrap().is_empty());
        assert!(!file_path.exists());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_v0_file_is_migrated_and_backed_up() {
        let (temp_dir, file_path) = temp_patients_file();
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(&file_path, V0_FILE).unwrap();

        let loaded = load_patients_from(&file_path).expect("Failed to load v0 file");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].antecedents, "Lombalgie chronique");
        assert_eq!(loaded[0].sessions[0].notes, "Patient motivé");

        let backup = temp_dir.join("patients.json.bak-v0");
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), V0_FILE);

        let rewritten: Value = serde_json::from_str(&std::fs::read_to_string(&file_path).unwrap()).unwrap();
        assert_eq!(rewritten["version"], CURRENT_VERSION);
        assert_eq!(load_patients_from(&file_path).unwrap(), loaded);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_newer_version_is_refused_and_untouched() {
        let (temp_dir, file_path) = temp_patients_file();
        std::fs::create_dir_all(&temp_dir).unwrap();
        let content = r#"{ "version": 99, "patients": [] }"#;
        std::fs::write(&file_path, content).unwrap();

        match load_patients_from(&file_path) {
            Err(LoadError::NewerVersion { found: 99, supported }) => assert_eq!(supported, CURRENT_VERSION),
            other => panic!("expected NewerVersion, got {other:?}"),
        }
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), content);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_corrupt_file_is_refused_and_untouched() {
        let (temp_dir, file_path) = temp_patients_file();
        std::fs::create_dir_all(&temp_dir).unwrap();
        let content = "{ pas du json";
        std::fs::write(&file_path, content).unwrap();

        assert!(matches!(load_patients_from(&file_path), Err(LoadError::Invalid(_))));
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), content);
        assert!(!temp_dir.join("patients.json.bak-v0").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_version() {
        assert_eq!(detect_version(&serde_json::json!([])).unwrap(), 0);
        assert_eq!(detect_version(&serde_json::json!({ "version": 1, "patients": [] })).unwrap(), 1);
        assert!(matches!(detect_version(&serde_json::json!({ "patients": [] })), Err(LoadError::Invalid(_))));
        assert!(matches!(detect_version(&serde_json::json!("texte")), Err(LoadError::Invalid(_))));
    }
}
