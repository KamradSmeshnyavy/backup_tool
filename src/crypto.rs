use crate::AppError;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore; // <-- добавлено для fill_bytes
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

#[derive(Serialize, Deserialize)]
pub struct EncryptedKey {
    pub ephemeral_public: [u8; 32],
    pub nonce: [u8; 12],
    pub encrypted_aes_key: Vec<u8>,
}

fn derive_aes_key_and_nonce(shared_secret: &[u8; 32]) -> ([u8; 32], [u8; 12]) {
    let hkdf = Hkdf::<Sha256>::new(None, shared_secret);
    let mut aes_key = [0u8; 32];
    let mut nonce = [0u8; 12];
    hkdf.expand(b"backup-tool aes encryption key", &mut aes_key)
        .expect("HKDF expand for key");
    hkdf.expand(b"backup-tool nonce for archive", &mut nonce)
        .expect("HKDF expand for nonce");
    (aes_key, nonce)
}

pub fn encrypt_backup(
    plaintext: &[u8],
    recipient_public_key: PublicKey,
) -> Result<(Vec<u8>, EncryptedKey), AppError> {
    let ephemeral_secret = EphemeralSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral_secret);
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_public_key);
    let (aes_key, nonce) = derive_aes_key_and_nonce(shared_secret.as_bytes());

    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .map_err(|e| AppError::Crypto(format!("Invalid AES key: {}", e)))?;
    let nonce = Nonce::from_slice(&nonce);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| AppError::Crypto(format!("Encryption failed: {}", e)))?;

    // Заворачивание сеансового ключа с отдельным nonce
    let mut wrap_nonce = [0u8; 12];
    OsRng.fill_bytes(&mut wrap_nonce); // теперь работает благодаря RngCore
    let wrap_cipher = Aes256Gcm::new_from_slice(&aes_key)
        .map_err(|e| AppError::Crypto(format!("Wrap cipher init: {}", e)))?;
    let encrypted_aes_key = wrap_cipher
        .encrypt(Nonce::from_slice(&wrap_nonce), aes_key.as_ref())
        .map_err(|e| AppError::Crypto(format!("Key wrapping failed: {}", e)))?;

    let envelope = EncryptedKey {
        ephemeral_public: *ephemeral_public.as_bytes(),
        nonce: wrap_nonce,
        encrypted_aes_key,
    };

    Ok((ciphertext, envelope))
}

pub fn decrypt_backup(
    ciphertext: &[u8],
    recipient_secret: &StaticSecret,
    envelope: &EncryptedKey,
) -> Result<Vec<u8>, AppError> {
    let ephemeral_public = PublicKey::from(envelope.ephemeral_public);
    let shared_secret = recipient_secret.diffie_hellman(&ephemeral_public);
    let (aes_key, archive_nonce) = derive_aes_key_and_nonce(shared_secret.as_bytes());

    let wrap_cipher = Aes256Gcm::new_from_slice(&aes_key)
        .map_err(|e| AppError::Crypto(format!("Wrap cipher init: {}", e)))?;
    let recovered_aes_key = wrap_cipher
        .decrypt(
            Nonce::from_slice(&envelope.nonce),
            envelope.encrypted_aes_key.as_ref(),
        )
        .map_err(|e| AppError::Crypto(format!("Key unwrapping failed: {}", e)))?;
    let recovered_key = <[u8; 32]>::try_from(recovered_aes_key.as_slice())
        .map_err(|_| AppError::Crypto("Invalid recovered key length".to_string()))?;

    let cipher = Aes256Gcm::new_from_slice(&recovered_key)
        .map_err(|e| AppError::Crypto(format!("Invalid recovered key: {}", e)))?;
    let nonce = Nonce::from_slice(&archive_nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Crypto(format!("Decryption failed: {}", e)))?;
    Ok(plaintext)
}

pub fn load_public_key(path: &std::path::Path) -> Result<PublicKey, AppError> {
    let bytes = std::fs::read(path)
        .map_err(|e| AppError::Config(format!("Cannot read public key: {}", e)))?;
    if bytes.len() != 32 {
        return Err(AppError::Config("Public key must be 32 bytes".to_string()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(PublicKey::from(arr))
}

pub fn load_secret_key(path: &std::path::Path) -> Result<StaticSecret, AppError> {
    let bytes = std::fs::read(path)
        .map_err(|e| AppError::Config(format!("Cannot read secret key: {}", e)))?;
    if bytes.len() != 32 {
        return Err(AppError::Config("Secret key must be 32 bytes".to_string()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(StaticSecret::from(arr))
}
