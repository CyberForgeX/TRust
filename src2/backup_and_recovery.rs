use tokio::time::{self, Duration};

impl DiskCache {
    async fn backup_to_disk(&self) {
        let backup_path = self.config.cache_dir.join("cache_backup.gz");
        let map = self.map.lock().await;
        let serialized_map = serde_json::to_vec(&*map).expect("Failed to serialize cache map");
        let encrypted_backup = self.encryption_service.encrypt(&serialized_map);

        tokio::fs::write(backup_path, encrypted_backup).await.expect("Failed to write backup");
    }

    async fn restore_from_backup(&self) {
        let backup_path = self.config.cache_dir.join("cache_backup.gz");
        let encrypted_backup = tokio::fs::read(backup_path).await.expect("Failed to read backup");
        let serialized_map = self.encryption_service.decrypt(&encrypted_backup);
        let map: HashMap<String, CacheEntry> = serde_json::from_slice(&serialized_map).expect("Failed to deserialize cache map");

        let mut current_map = self.map.lock().await;
        *current_map = map;
    }
}

async fn periodic_backup(storage: Arc<DiskCache>, interval: Duration) {
    let mut interval_timer = time::interval(interval);
    loop {
        interval_timer.tick().await;
        storage.backup_to_disk().await;
    }
}
