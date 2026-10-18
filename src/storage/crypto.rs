use argon2::Argon2;
use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 24;
pub type Key = [u8; 32];

pub fn derive_key(password: &str, salt: &[u8; SALT_LEN]) -> Result<Key, String> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|error| error.to_string())?;
    Ok(key)
}

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn seal(key: &Key, nonce: &[u8; NONCE_LEN], aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    XChaCha20Poly1305::new(key.into())
        .encrypt(XNonce::from_slice(nonce), Payload { msg: plaintext, aad })
        .map_err(|error| error.to_string())
}

pub fn open(key: &Key, nonce: &[u8; NONCE_LEN], aad: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    XChaCha20Poly1305::new(key.into())
        .decrypt(XNonce::from_slice(nonce), Payload { msg: ciphertext, aad })
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SALT: [u8; SALT_LEN] = [7; SALT_LEN];

    #[test]
    fn test_seal_and_open_roundtrip() {
        let key = derive_key("mot de passe", &SALT).unwrap();
        let nonce = random_bytes::<NONCE_LEN>();
        let sealed = seal(&key, &nonce, b"aad", b"patients").unwrap();
        assert_ne!(sealed, b"patients");
        assert_eq!(open(&key, &nonce, b"aad", &sealed).unwrap(), b"patients");
    }

    #[test]
    fn test_wrong_key_is_refused() {
        let key = derive_key("mot de passe", &SALT).unwrap();
        let other = derive_key("autre mot de passe", &SALT).unwrap();
        let nonce = random_bytes::<NONCE_LEN>();
        let sealed = seal(&key, &nonce, b"aad", b"patients").unwrap();
        assert!(open(&other, &nonce, b"aad", &sealed).is_err());
    }

    #[test]
    fn test_tampered_aad_or_ciphertext_is_refused() {
        let key = derive_key("mot de passe", &SALT).unwrap();
        let nonce = random_bytes::<NONCE_LEN>();
        let mut sealed = seal(&key, &nonce, b"aad", b"patients").unwrap();
        assert!(open(&key, &nonce, b"autre aad", &sealed).is_err());
        sealed[0] ^= 1;
        assert!(open(&key, &nonce, b"aad", &sealed).is_err());
    }
}
