impl DiskCache {
    async fn evict_if_necessary(&self) {
        let mut map = self.map.lock().await;
        if map.len() > self.config.cache_size {
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|&(_, entry)| (entry.access_count, entry.expiry));
            
            // Evict entries starting from the least accessed
            while map.len() > self.config.cache_size {
                if let Some((key, _)) = entries.pop() {
                    map.remove(key);
                }
            }
        }
    }
}
