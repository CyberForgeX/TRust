use tokio::sync::Mutex;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io;
use std::fs::File;
use std::io::{Read, Write};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::{Instant, SystemTime}};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    value: Vec<u8>, // Stored as encrypted data
    expiry: Option<Instant>,
    access_count: usize,
}

struct CacheMetrics {
    hits: u64,
    misses: u64,
    evictions: u64,
    // Add more metrics as needed
}


#[derive(Debug)]
enum CacheError {
    IoError(io::Error),
    SerializationError(bincode::Error),
    EncryptionError(String), // Assuming encryption can fail in specific ways
}

impl From<io::Error> for CacheError {
    fn from(err: io::Error) -> Self {
        CacheError::IoError(err)
    }
}

impl From<bincode::Error> for CacheError {
    fn from(err: bincode::Error) -> Self {
        CacheError::SerializationError(err)
    }
}

struct Config {
    encryption_key_path: PathBuf,
    cache_size: usize,
    eviction_policy: CacheEvictionPolicy,
}

struct EncryptionService {
    // Encryption logic here
}

#[async_trait]
trait Storage {
    async fn set(&self, key: String, value: Vec<u8>, ttl: Option<Instant>);
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    async fn cleanup(&self);
}

enum CacheEvictionPolicy {
    LRU,
    LFU,
    // Add more strategies as needed
}



impl EncryptionService {
    fn new(key_path: &PathBuf) -> Self {
        // Initialize encryption service
        EncryptionService {
            // Encryption initialization logic
        }
    }

    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        // Encrypt data
        vec![] // Placeholder
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        // Decrypt data
        vec![] // Placeholder
    }
}

struct DiskCache {
    map: Arc<Mutex<HashMap<String, CacheEntry>>>,
    config: Config,
    encryption_service: EncryptionService,
}

impl DiskCache {
    async fn new(config: Config) -> Self {
        let encryption_service = EncryptionService::new(&config.encryption_key_path);
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
            config,
            encryption_service,
        }
    }

    async fn load_from_disk(&self) {
        let path = &self.config.encryption_key_path.join("cache_data.bin");
        if path.exists() {
            let mut file = File::open(path).expect("Failed to open cache file");
            let mut encrypted_data = Vec::new();
            file.read_to_end(&mut encrypted_data).expect("Failed to read cache file");
            let decrypted_data = self.encryption_service.decrypt(&encrypted_data);
            let cache_map: HashMap<String, CacheEntry> = bincode::deserialize(&decrypted_data).expect("Failed to deserialize cache data");
            
            let mut map = self.map.lock().await;
            *map = cache_map;
        }
    }

    async fn save_to_disk(&self) {
        let path = &self.config.encryption_key_path.join("cache_data.bin");
        let map = self.map.lock().await;
        let serialized_data = bincode::serialize(&*map).expect("Failed to serialize cache data");
        let encrypted_data = self.encryption_service.encrypt(&serialized_data);
        
        let mut file = File::create(path).expect("Failed to create cache file");
        file.write_all(&encrypted_data).expect("Failed to write encrypted cache data");
    }
    
}

#[async_trait]
impl Storage for DiskCache {
    async fn set(&self, key: String, value: Vec<u8>, ttl: Option<Instant>) {
        let encrypted_value = self.encryption_service.encrypt(&value);
        let entry = CacheEntry {
            value: encrypted_value,
            expiry: ttl,
            access_count: 0,
        };

        let mut map = self.map.lock().await;
        map.insert(key, entry);

        // Optionally, implement logic to prevent cache from growing indefinitely
        if map.len() > self.config.cache_size {
            self.cleanup_least_frequently_used().await;
        }
    }

    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut map = self.map.lock().await;
        map.get_mut(key).and_then(|entry| {
            entry.access_count += 1;
            entry.expiry.and_then(|expiry| {
                if expiry > Instant::now() {
                    Some(self.encryption_service.decrypt(&entry.value))
                } else {
                    map.remove(key);
                    None
                }
            })
        })
    }

    async fn cleanup(&self) {
        let mut map = self.map.lock().await;
        map.retain(|_, entry| {
            entry.expiry.map_or(true, |expiry| expiry > Instant::now())
        });
    }

    async fn cleanup_least_frequently_used(&self) {
        let mut map = self.map.lock().await;
        if map.len() <= self.config.cache_size {
            return;
        }
    
        let mut entries: Vec<_> = map.iter().collect();
        // Sort by access count, ascending (least frequently accessed first)
        entries.sort_by_key(|(_, entry)| entry.access_count);
    
        while map.len() > self.config.cache_size && !entries.is_empty() {
            if let Some((key, _)) = entries.remove(0) {
                map.remove(key);
            }
        }
    }
}
