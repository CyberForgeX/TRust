use std::{
    collections::{BinaryHeap, HashMap},
    fs,
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
    cmp::Ordering,
};

use aes_gcm::{aead::Aead, Aes256Gcm};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{Mutex, Semaphore},
};
use std::time::Duration;

const CACHE_DIR: &str = "cache_dir";
const CONFIG_FILE: &str = "config.json";
const ENCRYPTION_KEY_FILE: &str = "encryption_key.bin";
const CACHE_FILE: &str = "cache.json.gz";
const BACKUP_FILE: &str = "cache_backup.json.gz";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    value: String,
    expiry: Option<Instant>,
    access_count: usize,
}

#[derive(Debug)]
enum CacheError {
    CleanupError(String),
    IoError(io::Error),
    EncryptionError,
    DecryptionError,
    NotFound,
    SerializationError(serde_json::Error),
    DeserializationError(serde_json::Error),
    IntegrityError,
}

struct DiskCache {
    map: Mutex<HashMap<String, CacheEntry>>,
    write_semaphore: Semaphore,
    encryption_enabled: bool,
    cache_dir: PathBuf,
    max_cache_size: usize,
    encryption_key: Option<Vec<u8>>,
}

impl DiskCache {
    async fn new(cache_dir: &str, initial_cache_size: usize, encryption_enabled: bool) -> Result<Self, CacheError> {
        let cache_dir = Path::new(cache_dir).to_path_buf();
        fs::create_dir_all(&cache_dir).map_err(|e| CacheError::IoError(e))?;

        let map = Mutex::new(HashMap::new());
        let write_semaphore = Semaphore::new(initial_cache_size);
        let encryption_key = if encryption_enabled {
            Some(Self::load_or_generate_encryption_key().await?)
        } else {
            None
        };

        let disk_cache = DiskCache {
            map,
            write_semaphore,
            encryption_enabled,
            cache_dir: cache_dir.clone(),
            max_cache_size: initial_cache_size,
            encryption_key,
        };
        disk_cache.load_from_disk().await?; // Load existing cache
        Ok(disk_cache)
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<(), CacheError> {
        let mut map = self.map.lock().await;
        map.insert(
            key.to_string(),
            CacheEntry {
                value: if let Some(encryption_key) = &self.encryption_key {
                    self.encrypt(value.as_bytes(), encryption_key).await?
                } else {
                    value.to_string()
                },
                expiry: ttl.map(|d| Instant::now() + d),
                access_count: 0,
            },
        );
        self.evict_if_necessary().await; // Evict if cache exceeds max size
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let mut map = self.map.lock().await;
        match map.get_mut(key) {
            Some(entry) => {
                entry.access_count += 1;
                if let Some(expiry) = entry.expiry {
                    if expiry <= Instant::now() {
                        return Ok(None); // Entry expired
                    }
                }
                if let Some(encryption_key) = &self.encryption_key {
                    Ok(Some(self.decrypt(&entry.value, encryption_key).await?))
                } else {
                    Ok(Some(entry.value.clone()))
                }
            }
            None => Ok(None), // Key not found
        }
    }

    async fn load_from_disk(&self) -> Result<(), CacheError> {
        let cache_file_path = self.cache_dir.join(CACHE_FILE);
        let cache_file = match File::open(&cache_file_path).await {
            Ok(file) => file,
            Err(_) => return Ok(()), // No cache file found
        };
        let mut decoder = GzDecoder::new(cache_file);
        let mut json_string = String::new();
        decoder.read_to_string(&mut json_string).await.map_err(CacheError::IoError)?;

        let cache_map: HashMap<String, CacheEntry> =
            serde_json::from_str(&json_string).map_err(CacheError::DeserializationError)?;

        let mut map = self.map.lock().await;
        *map = cache_map;

        if let Some(encryption_key) = &self.encryption_key {
            // Verify integrity of cache entries
            for entry in map.values() {
                if let Err(_) = self.decrypt(&entry.value, encryption_key).await {
                    return Err(CacheError::IntegrityError);
                }
            }
        }

        Ok(())
    }

    async fn save_to_disk(&self) -> Result<(), CacheError> {
        let cache_file_path = self.cache_dir.join(CACHE_FILE);
        let cache_map = self.map.lock().await.clone();
        let json_string = serde_json::to_string(&cache_map).map_err(CacheError::SerializationError)?;

        let cache_file = File::create(&cache_file_path).await.map_err(CacheError::IoError)?;
        let mut encoder = GzEncoder::new(cache_file, Compression::default());
        encoder.write_all(json_string.as_bytes()).await.map_err(CacheError::IoError)?;

        Ok(())
    }

    async fn backup(&self) -> Result<(), CacheError> {
        let cache_file_path = self.cache_dir.join(CACHE_FILE);
        let backup_file_path = self.cache_dir.join(BACKUP_FILE);
        fs::copy(&cache_file_path, &backup_file_path).map_err(CacheError::IoError)?;
        Ok(())
    }

    async fn restore_backup(&self) -> Result<(), CacheError> {
        let cache_file_path = self.cache_dir.join(CACHE_FILE);
        let backup_file_path = self.cache_dir.join(BACKUP_FILE);
        fs::copy(&backup_file_path, &cache_file_path).map_err(CacheError::IoError)?;
        Ok(())
    }

    async fn clean_cache(&self) -> Result<(), CacheError> {
        let cache_file_path = self.cache_dir.join(CACHE_FILE);
        fs::remove_file(&cache_file_path).or_else(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(CacheError::CleanupError(format!("Failed to clean cache: {}", err)))
            }
        })?;
        Ok(())
    }

    async fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<String, CacheError> {
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = aes_gcm::Nonce::generate();
        let ciphertext = cipher.encrypt(&nonce, data).map_err(|_| CacheError::EncryptionError)?;
        Ok(base64::encode(&nonce.to_bytes()) + &base64::encode(&ciphertext))
    }

    async fn decrypt(&self, data: &str, key: &[u8]) -> Result<String, CacheError> {
        let cipher = Aes256Gcm::new(&key.into());
        let nonce_length = aes_gcm::Nonce::SIZE;
        let nonce_data = base64::decode(&data[..nonce_length]).map_err(|_| CacheError::DecryptionError)?;
        let nonce = aes_gcm::Nonce::from_slice(&nonce_data);
        let ciphertext = base64::decode(&data[nonce_length..]).map_err(|_| CacheError::DecryptionError)?;
        let plaintext = cipher.decrypt(nonce.unwrap(), &ciphertext).map_err(|_| CacheError::DecryptionError)?;
        String::from_utf8(plaintext).map_err(|_| CacheError::DecryptionError)
    }

    async fn evict_if_necessary(&self) {
        // Acquire a non-blocking lock to check the cache size and perform eviction if necessary
        if let Some(mut guard) = self.write_semaphore.try_acquire_owned() {
            let mut map = self.map.lock().await;
            if map.len() > self.max_cache_size {
                // Count accesses for each entry
                let mut access_counts: HashMap<&String, usize> = HashMap::new();
                for (key, entry) in map.iter() {
                    access_counts.insert(key, entry.access_count);
                }
                // Sort entries by access count, breaking ties by expiry time
                let mut entries: Vec<(&String, &CacheEntry)> = map.iter().collect();
                entries.sort_by(|a, b| {
                    let count_cmp = access_counts.get(b.0).unwrap().cmp(access_counts.get(a.0).unwrap());
                    if count_cmp == Ordering::Equal {
                        let expiry_a = a.1.expiry.unwrap_or(Instant::now());
                        let expiry_b = b.1.expiry.unwrap_or(Instant::now());
                        expiry_b.cmp(&expiry_a)
                    } else {
                        count_cmp
                    }
                });
                // Remove the least frequently accessed entries until cache size is within limits
                for (key, _) in entries.into_iter().take(map.len() - self.max_cache_size) {
                    map.remove(&key);
                }
            }
        } else {
            // Failed to acquire the semaphore, indicating high contention
            // Increase the maximum cache size or implement a more efficient eviction strategy
            println!("Failed to acquire write semaphore for eviction, consider increasing cache size or optimizing eviction strategy.");
        }
    }

    async fn load_or_generate_encryption_key() -> Result<Vec<u8>, CacheError> {
        let key_file_path = Path::new(ENCRYPTION_KEY_FILE);
        if key_file_path.exists() {
            fs::read(key_file_path).map_err(|e| CacheError::IoError(e))
        } else {
            let mut key = vec![0u8; 32]; // 256-bit key
            OsRng.fill_bytes(&mut key); // Fill key with random bytes
            fs::write(key_file_path, &key).map_err(|e| CacheError::IoError(e))?;
            Ok(key)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    encryption_enabled: bool,
    auto_update_enabled: bool,
}

impl Config {
    async fn load() -> Result<Self, CacheError> {
        let config_file_path = Path::new(CONFIG_FILE);
        if config_file_path.exists() {
            let config_str = fs::read_to_string(config_file_path)
            .map_err(CacheError::IoError)?;
        serde_json::from_str(&config_str)
            .map_err(CacheError::DeserializationError)
    } else {
        let default_config = Config {
            encryption_enabled: true,
            auto_update_enabled: true,
        };
        let default_config_str = serde_json::to_string(&default_config)
            .map_err(CacheError::SerializationError)?;
        fs::write(config_file_path, default_config_str)
            .map_err(CacheError::IoError)?;
        Ok(default_config)
    }
}

async fn save(&self) -> Result<(), CacheError> {
    let config_file_path = Path::new(CONFIG_FILE);
    let config_str = serde_json::to_string(self)
        .map_err(CacheError::SerializationError)?;
    fs::write(config_file_path, config_str)
        .map_err(CacheError::IoError)?;
    Ok(())
}
}

#[tokio::main]
async fn main() -> Result<(), CacheError> {
    println!("Cleaning cache...");
    let cache = DiskCache::new(CACHE_DIR, 100, true).await?;
    cache.clean_cache().await?;
    println!("Cache cleaned.");

    println!("Building cache...");
    let config = Config::load().await?;
    let cache = DiskCache::new(CACHE_DIR, 100, config.encryption_enabled).await?;
    cache.load_from_disk().await?;
    println!("Cache built.");

    let key = "test_key";
    let value = "test_value";
    println!("Setting key-value pair...");
    cache.set(key, value, Some(Duration::minutes(5))).await?;
    println!("Key-value pair set successfully.");

    println!("Retrieving value for key...");
    if let Some(val) = cache.get(key).await? {
        println!("Retrieved value from cache: {}", val);
    } else {
        println!("Value not found in cache");
    }

    println!("Saving cache to disk...");
    cache.save_to_disk().await?;
    println!("Cache saved to disk.");

    println!("Backing up cache...");
    cache.backup().await?;
    println!("Cache backed up successfully.");

    println!("Restoring cache from backup...");
    cache.restore_backup().await?;
    println!("Cache restored from backup.");

    Ok(())
}
