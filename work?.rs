use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use generic_array::typenum::U12;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    process::Command,
};
use thiserror::Error;
use tokio::{
    fs::File as AsyncFile,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Semaphore,
};
use chrono::{Duration, Utc};
use dialoguer::{Input, Confirm};

// Constants
const CACHE_DIR: &str = "/tmp/rust_cache";
const MAX_CONCURRENT_WRITES: usize = 10;
const DEFAULT_CACHE_CAPACITY: usize = 100;
const ENCRYPTION_KEY_FILE: &str = "/tmp/encryption_key";

// Custom Error Type for Cache Operations
#[derive(Debug, Error)]
enum CacheError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization Error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Encryption Error: {0}")]
    EncryptionError(String),
    #[error("Decryption Error: {0}")]
    DecryptionError(String),
    #[error("Access Denied")]
    AccessDenied,
    #[error("Compression Error: {0}")]
    CompressionError(String),
    #[error("Decompression Error: {0}")]
    DecompressionError(String),
}

// Cache Entry Structure
#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    value: String,
    expiry: Option<i64>,
    checksum: String,
    nonce: Option<Nonce>,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.expiry.map_or(false, |expiry| expiry <= Utc::now().timestamp())
    }
}

// Thread-safe Disk Cache Implementation
struct DiskCache {
    dir: PathBuf,
    map: Arc<Mutex<HashMap<String, CacheEntry>>>,
    capacity: usize,
    write_semaphore: Arc<Semaphore>,
    encryption_enabled: bool,
}

impl DiskCache {
    async fn new(cache_dir: &str, capacity: usize, encryption_enabled: bool) -> Result<Self, CacheError> {
        let dir = PathBuf::from(cache_dir);
        fs::create_dir_all(&dir)?;

        let map = Arc::new(Mutex::new(HashMap::new()));
        let write_semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_WRITES));

        Ok(Self {
            dir,
            map,
            capacity,
            write_semaphore,
            encryption_enabled,
        })
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>, use_compression: bool) -> Result<(), CacheError> {
        let expiry = ttl.map(|duration| Utc::now().timestamp() + duration.num_seconds());
        let nonce = if self.encryption_enabled { Some(Nonce::generate()) } else { None };
        let entry = CacheEntry { 
            value: value.to_owned(), 
            expiry, 
            checksum: hash_value(value),
            nonce,
        };

        let mut map = self.map.lock().await;
        map.insert(key.to_owned(), entry.clone());

        let file_path = self.dir.join(key);
        let permit = self.write_semaphore.acquire().await?;
        let mut file = AsyncFile::create(file_path).await?;
        let serialized_entry = serde_json::to_string(&entry)?;
        let encrypted_entry = if let Some(nonce) = &entry.nonce {
            encrypt(serialized_entry, &nonce)?
        } else {
            serialized_entry.into_bytes()
        };
        file.write_all(&encrypted_entry).await?;
        drop(permit);

        Ok(())
    }

    async fn get(&self, key: &str, use_compression: bool) -> Result<Option<String>, CacheError> {
        let map = self.map.lock().await;
        if let Some(entry) = map.get(key) {
            if entry.is_expired() {
                return Ok(None);
            }
            return Ok(Some(entry.value.clone()));
        }

        let file_path = self.dir.join(key);
        if file_path.exists() {
            let mut file = AsyncFile::open(file_path).await?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).await?;

            let decrypted_entry = if self.encryption_enabled {
                let entry: CacheEntry = serde_json::from_slice(&contents)?;
                let nonce = entry.nonce.ok_or(CacheError::DecryptionError("Nonce missing".to_string()))?;
                encrypt(contents, &nonce)?
            } else {
                contents
            };

            let entry: CacheEntry = serde_json::from_slice(&decrypted_entry)?;
            if entry.is_expired() {
                return Ok(None);
            }

            let calculated_checksum = hash_value(&entry.value);
            if entry.checksum != calculated_checksum {
                return Err(CacheError::DecompressionError(format!("Checksum mismatch for key {}", key)));
            }

            return Ok(Some(entry.value));
        }

        Ok(None)
    }
}

fn generate_nonce() -> Nonce {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    Nonce::from_slice(&nonce)
}

fn encrypt(data: &str, nonce: &Nonce) -> Result<Vec<u8>, CacheError> {
    let key = load_encryption_key()?;
    let cipher = Aes256Gcm::new(Key::from_slice(&key));
    cipher.encrypt(nonce, data.as_bytes())
        .map_err(|_| CacheError::EncryptionError("Encryption failed".into()))
}

fn load_encryption_key() -> Result<Key, CacheError> {
    if PathBuf::from(ENCRYPTION_KEY_FILE).exists() {
        let mut file = File::open(ENCRYPTION_KEY_FILE)?;
        let mut key_bytes = [0u8; 32];
        file.read_exact(&mut key_bytes)?;
        Ok(Key::from_slice(&key_bytes))
    } else {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let mut file = File::create(ENCRYPTION_KEY_FILE)?;
        file.write_all(&key_bytes)?;
        Ok(Key::from_slice(&key_bytes))
    }
}

fn compress(data: &[u8]) -> Result<Vec<u8>, CacheError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish().map_err(|e| CacheError::CompressionError(e.to_string()))
}

fn decompress(data: &[u8]) -> Result<Vec<u8>, CacheError> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}

fn hash_value(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    let result = hasher.finalize();
    hex::encode(result)
}

fn load_encryption_key() -> Result<Key, CacheError> {
    if Path::new(ENCRYPTION_KEY_FILE).exists() {
        let mut file = File::open(ENCRYPTION_KEY_FILE)?;
        let mut key_bytes = [0u8; 32];
        file.read_exact(&mut key_bytes)?;
        Ok(Key::from_slice(&key_bytes))
    } else {
        let key = Key::generate();
        let mut file = File::create(ENCRYPTION_KEY_FILE)?;
        file.write_all(key.as_ref())?;
        Ok(key)
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    encryption_enabled: bool,
}

fn load_config() -> Result<Config, CacheError> {
    Ok(Config {
        encryption_enabled: env::var("ENCRYPTION_ENABLED").unwrap_or_else(|_| "false".to_string()).parse().unwrap(),
    })
}

#[tokio::main]
async fn main() -> Result<(), CacheError> {
    let exec_path = env::current_exe()?.to_string_lossy().into_owned();
    println!("Detected executable path: {}", exec_path);

    let service_name: String = Input::new()
        .with_prompt("Enter the service name")
        .default("my_rust_service".into())
        .interact_text()?;

    let description: String = Input::new()
        .with_prompt("Enter a description for your service")
        .default("My Rust Service".into())
        .interact_text()?;

    let additional_env: String = Input::new()
        .with_prompt("Enter additional environment variables (key=value), separate multiple with ';'")
        .default("".into())
        .interact_text()?;

    let additional_args: String = Input::new()
        .with_prompt("Enter additional command-line arguments for your application")
        .default("".into())
        .interact_text()?;

    let service_file_content = format!(
        "[Unit]\n\
         Description={description}\n\
         After=network.target\n\
         \n\
         [Service]\n\
         Environment=\"ENCRYPTION_ENABLED=true\"{additional_env}\n\
         ExecStart={exec_path} {additional_args}\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n",
        description=description,
        exec_path=exec_path,
        additional_env=additional_env.split(';').map(|s| format!("\nEnvironment=\"{}\"", s)).collect::<String>(),
        additional_args=additional_args,
    );

    let service_file_name = format!("/etc/systemd/system/{}.service", service_name);
    let service_file_path = Path::new(&service_file_name);

    let service_file_path = Path::new(&format!("/etc/systemd/system/{}.service", service_name));
    let mut file = File::create(service_file_path)?;
    file.write_all(service_file_content.as_bytes())?;
    println!("Systemd service file created at: {}", service_file_path.display());

    if Confirm::new().with_prompt("Would you like to reload systemd daemons and enable the service now?").interact()? {
        Command::new("systemctl").arg("daemon-reload").status()?;
        Command::new("systemctl").arg("enable").arg(&service_name).status()?;
        println!("Service '{}' enabled. You can start it with 'systemctl start {}'", service_name, service_name);
    }

    Ok(())
}
