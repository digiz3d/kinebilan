mod crypto;
mod migrations;

use std::path::{Path, PathBuf};

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::patient::Patient;

pub const CURRENT_VERSION: u32 = 1;
const KDF: &str = "argon2id";

#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    Invalid(String),
    NewerVersion { found: u32, supported: u32 },
    WrongPassword,
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
            Self::WrongPassword => {
                write!(f, "Mot de passe incorrect, ou fichier de sauvegarde altéré.")
            }
        }
    }
}

/// Key material for the open save file. Kept in memory while the app runs.
pub struct Vault {
    key: crypto::Key,
    salt: [u8; crypto::SALT_LEN],
}

impl Vault {
    fn derive(password: &str, salt: [u8; crypto::SALT_LEN]) -> Result<Self, LoadError> {
        let key = crypto::derive_key(password, &salt).map_err(LoadError::Invalid)?;
        Ok(Self { key, salt })
    }

    fn new(password: &str) -> Result<Self, LoadError> {
        Self::derive(password, crypto::random_bytes())
    }
}

pub enum Stored {
    Empty,
    Plaintext,
    Encrypted,
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    patients: Vec<Patient>,
}

#[derive(Serialize)]
struct PayloadRef<'a> {
    patients: &'a [Patient],
}

#[derive(Serialize, Deserialize)]
struct Envelope {
    version: u32,
    encryption: Encryption,
    data: String,
}

#[derive(Serialize, Deserialize)]
struct Encryption {
    kdf: String,
    salt: String,
    nonce: String,
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

fn read_raw(path: &Path) -> Result<Option<Value>, LoadError> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(LoadError::Io(error)),
    };
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|error| LoadError::Invalid(error.to_string()))
}

fn stored_from(path: &Path) -> Result<Stored, LoadError> {
    let Some(raw) = read_raw(path)? else {
        return Ok(Stored::Empty);
    };
    if raw.get("encryption").is_some() {
        Ok(Stored::Encrypted)
    } else {
        Ok(Stored::Plaintext)
    }
}

/// Reads an unencrypted save file. Older versions are backed up before migrating.
fn load_plaintext_from(path: &Path) -> Result<Vec<Patient>, LoadError> {
    let Some(raw) = read_raw(path)? else {
        return Ok(Vec::new());
    };
    let version = detect_version(&raw)?;
    if version > CURRENT_VERSION {
        return Err(LoadError::NewerVersion { found: version, supported: CURRENT_VERSION });
    }
    if version < CURRENT_VERSION {
        let backup = path.with_extension(format!("json.bak-v{version}"));
        std::fs::copy(path, &backup).map_err(LoadError::Io)?;
    }
    migrate(raw, version)
}

fn aad(version: u32, kdf: &str, salt: &str) -> String {
    format!("kinebilan|{version}|{kdf}|{salt}")
}

fn decode_array<const N: usize>(text: &str) -> Result<[u8; N], LoadError> {
    let bytes = BASE64
        .decode(text)
        .map_err(|error| LoadError::Invalid(error.to_string()))?;
    bytes
        .try_into()
        .map_err(|_| LoadError::Invalid("taille de champ invalide".into()))
}

fn encode(vault: &Vault, patients: &[Patient]) -> Result<String, LoadError> {
    let plaintext = serde_json::to_vec(&PayloadRef { patients })
        .map_err(|error| LoadError::Invalid(error.to_string()))?;
    let nonce = crypto::random_bytes::<{ crypto::NONCE_LEN }>();
    let salt = BASE64.encode(vault.salt);
    let ciphertext = crypto::seal(&vault.key, &nonce, aad(CURRENT_VERSION, KDF, &salt).as_bytes(), &plaintext)
        .map_err(LoadError::Invalid)?;
    let envelope = Envelope {
        version: CURRENT_VERSION,
        encryption: Encryption { kdf: KDF.into(), salt, nonce: BASE64.encode(nonce) },
        data: BASE64.encode(ciphertext),
    };
    serde_json::to_string_pretty(&envelope).map_err(|error| LoadError::Invalid(error.to_string()))
}

fn decode(vault: &Vault, raw: Value) -> Result<Vec<Patient>, LoadError> {
    let version = detect_version(&raw)?;
    if version > CURRENT_VERSION {
        return Err(LoadError::NewerVersion { found: version, supported: CURRENT_VERSION });
    }
    let envelope: Envelope =
        serde_json::from_value(raw).map_err(|error| LoadError::Invalid(error.to_string()))?;
    if envelope.encryption.kdf != KDF {
        return Err(LoadError::Invalid(format!("méthode de chiffrement inconnue : {}", envelope.encryption.kdf)));
    }
    let nonce = decode_array::<{ crypto::NONCE_LEN }>(&envelope.encryption.nonce)?;
    let ciphertext = BASE64
        .decode(&envelope.data)
        .map_err(|error| LoadError::Invalid(error.to_string()))?;
    let aad = aad(version, &envelope.encryption.kdf, &envelope.encryption.salt);
    let plaintext = crypto::open(&vault.key, &nonce, aad.as_bytes(), &ciphertext)
        .map_err(|_| LoadError::WrongPassword)?;

    let mut payload: Value =
        serde_json::from_slice(&plaintext).map_err(|error| LoadError::Invalid(error.to_string()))?;
    let object = payload
        .as_object_mut()
        .ok_or_else(|| LoadError::Invalid("contenu chiffré inattendu".into()))?;
    object.insert("version".into(), json!(version));
    migrate(payload, version)
}

fn write_atomic(path: &Path, data: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)
}

fn unlock_from(path: &Path, password: &str) -> Result<(Vault, Vec<Patient>), LoadError> {
    let raw = read_raw(path)?
        .ok_or_else(|| LoadError::Invalid("fichier de sauvegarde introuvable".into()))?;
    let envelope: Envelope =
        serde_json::from_value(raw.clone()).map_err(|error| LoadError::Invalid(error.to_string()))?;
    let salt = decode_array::<{ crypto::SALT_LEN }>(&envelope.encryption.salt)?;
    let vault = Vault::derive(password, salt)?;
    let patients = decode(&vault, raw)?;
    Ok((vault, patients))
}

/// Starts encryption for a new or unencrypted save file. The result is written encrypted,
/// after checking that it decrypts back to the same patients.
fn create_from(path: &Path, password: &str) -> Result<(Vault, Vec<Patient>), LoadError> {
    let patients = match stored_from(path)? {
        Stored::Encrypted => {
            return Err(LoadError::Invalid("le fichier est déjà chiffré".into()));
        }
        Stored::Plaintext => load_plaintext_from(path)?,
        Stored::Empty => Vec::new(),
    };
    let vault = Vault::new(password)?;
    let data = encode(&vault, &patients)?;
    let raw: Value = serde_json::from_str(&data).map_err(|error| LoadError::Invalid(error.to_string()))?;
    if decode(&vault, raw)? != patients {
        return Err(LoadError::Invalid("la vérification du chiffrement a échoué".into()));
    }
    write_atomic(path, &data).map_err(LoadError::Io)?;
    Ok((vault, patients))
}

fn save_to(path: &Path, vault: &Vault, patients: &[Patient]) -> std::io::Result<()> {
    let data = encode(vault, patients).map_err(|error| std::io::Error::other(error.to_string()))?;
    write_atomic(path, &data)
}

pub fn stored_kind() -> Result<Stored, LoadError> {
    stored_from(&storage_path())
}

pub fn unlock(password: &str) -> Result<(Vault, Vec<Patient>), LoadError> {
    unlock_from(&storage_path(), password)
}

pub fn create_vault(password: &str) -> Result<(Vault, Vec<Patient>), LoadError> {
    create_from(&storage_path(), password)
}

pub fn save_patients(vault: &Vault, patients: &[Patient]) -> std::io::Result<()> {
    save_to(&storage_path(), vault, patients)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patient::Session;

    const PASSWORD: &str = "mot de passe de test";

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
    fn test_save_and_unlock_patients() {
        let (temp_dir, file_path) = temp_patients_file();
        let p1 = sample_patient();
        let (vault, _) = create_from(&file_path, PASSWORD).expect("Failed to create");

        save_to(&file_path, &vault, std::slice::from_ref(&p1)).expect("Failed to save");

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(!content.contains("Dupont"));
        let raw: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(raw["version"], CURRENT_VERSION);
        assert!(raw.get("patients").is_none());

        let (_, loaded) = unlock_from(&file_path, PASSWORD).expect("Failed to unlock");
        assert_eq!(loaded, vec![p1]);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_missing_file_is_empty() {
        let (temp_dir, file_path) = temp_patients_file();
        assert!(matches!(stored_from(&file_path), Ok(Stored::Empty)));
        let (_, patients) = create_from(&file_path, PASSWORD).unwrap();
        assert!(patients.is_empty());
        assert!(matches!(stored_from(&file_path), Ok(Stored::Encrypted)));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_v0_file_is_encrypted_and_backed_up() {
        let (temp_dir, file_path) = temp_patients_file();
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(&file_path, V0_FILE).unwrap();

        let (_, loaded) = create_from(&file_path, PASSWORD).expect("Failed to convert v0 file");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].antecedents, "Lombalgie chronique");
        assert_eq!(loaded[0].sessions[0].notes, "Patient motivé");

        let backup = temp_dir.join("patients.json.bak-v0");
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), V0_FILE);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(!content.contains("Dupont"));
        assert!(matches!(stored_from(&file_path), Ok(Stored::Encrypted)));
        assert_eq!(unlock_from(&file_path, PASSWORD).unwrap().1, loaded);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_wrong_password_is_refused_and_untouched() {
        let (temp_dir, file_path) = temp_patients_file();
        let (vault, _) = create_from(&file_path, PASSWORD).unwrap();
        save_to(&file_path, &vault, &[sample_patient()]).unwrap();
        let content = std::fs::read_to_string(&file_path).unwrap();

        assert!(matches!(unlock_from(&file_path, "mauvais"), Err(LoadError::WrongPassword)));
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), content);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_tampered_version_is_refused() {
        let (temp_dir, file_path) = temp_patients_file();
        let (vault, _) = create_from(&file_path, PASSWORD).unwrap();
        save_to(&file_path, &vault, &[sample_patient()]).unwrap();

        let mut raw: Value = serde_json::from_str(&std::fs::read_to_string(&file_path).unwrap()).unwrap();
        raw["version"] = json!(0);
        std::fs::write(&file_path, raw.to_string()).unwrap();

        assert!(matches!(unlock_from(&file_path, PASSWORD), Err(LoadError::WrongPassword)));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_newer_version_is_refused_and_untouched() {
        let (temp_dir, file_path) = temp_patients_file();
        std::fs::create_dir_all(&temp_dir).unwrap();
        let content = r#"{ "version": 99, "patients": [] }"#;
        std::fs::write(&file_path, content).unwrap();

        match create_from(&file_path, PASSWORD) {
            Err(LoadError::NewerVersion { found: 99, supported }) => assert_eq!(supported, CURRENT_VERSION),
            Err(_) => panic!("expected NewerVersion"),
            Ok(_) => panic!("expected an error"),
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

        assert!(matches!(create_from(&file_path, PASSWORD), Err(LoadError::Invalid(_))));
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
