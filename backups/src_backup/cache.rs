use std::collections::HashMap;
use std::time::{Instant, Duration};
use tokio::sync::Mutex;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Debug, Clone)]
struct CacheEntry {
    // Represents the data stored in your cache
    data: String, // Simplified to store just a string for demonstration
}

#[async_trait]
trait Cache {
    async fn set(&self, key: String, value: CacheEntry);
    async fn get(&self, key: &str) -> Option<CacheEntry>;
}

struct SimpleCache {
    store: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl SimpleCache {
    fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Cache for SimpleCache {
    async fn set(&self, key: String, value: CacheEntry) {
        let mut store = self.store.lock().await;
        store.insert(key, value);
    }

    async fn get(&self, key: &str) -> Option<CacheEntry> {
        let store = self.store.lock().await;
        store.get(key).cloned()
    }
}

// Function to simulate loading data for a given key
async fn load_data_for_key(key: &str) -> CacheEntry {
    // In a real application, this would involve fetching data from a database, performing some computation, etc.
    CacheEntry { data: format!("Data for {}", key) }
}
