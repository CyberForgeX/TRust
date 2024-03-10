use aes_gcm::{Aes256Gcm, aead::{Aead, NewAead, generic_array::GenericArray}};
use aes_gcm::aead::Error as AeadError;
use rand::{rngs::OsRng, RngCore};
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{Read, Write};

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    AeadError(AeadError),
}

impl From<std::io::Error> for EncryptionError {
    fn from(error: std::io::Error) -> Self {
        EncryptionError::IoError(error)
    }
}

impl From<AeadError> for EncryptionError {
    fn from(error: AeadError) -> Self {
        EncryptionError::AeadError(error)
    }
}

fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    async fn new(key_path: &PathBuf) -> Result<Self, EncryptionError> {
        let key = if key_path.exists() {
            tokio::fs::read(key_path).await? // Ensure .await is used
        } else {
            let mut key = vec![0u8; 32];
            OsRng.fill_bytes(&mut key);
            tokio::fs::write(key_path, &key).await?; // Ensure .await is used
            key
        };
        let cipher = Aes256Gcm::new_from_slice(&key)?;
        Ok(EncryptionService { cipher })
    }
}


