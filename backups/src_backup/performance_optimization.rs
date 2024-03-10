struct PerformanceOptimizer {
    prefetch_list: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl PerformanceOptimizer {
    fn new() -> Self {
        Self {
            prefetch_list: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn prefetch_data(&self, storage: Arc<DiskCache>, keys: Vec<String>) {
        for key in keys.iter() {
            if let Some(data) = storage.get(key).await {
                let mut list = self.prefetch_list.write().await; // Ensure async lock
                list.insert(key.clone(), data);
            }
        }
    }
    

    async fn get_prefetched_data(&self, key: &str) -> Option<Vec<u8>> {
        let list = self.prefetch_list.read().await;
        list.get(key).cloned()
    }
}
