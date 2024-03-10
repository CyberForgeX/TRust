#[derive(Debug, Clone)]
struct AccessRecord {
    last_access: Instant,
    frequency: Duration,
}

#[derive(Debug, Clone)]
struct Prefetcher {
    access_records: Arc<Mutex<HashMap<String, AccessRecord>>>,
    cache: Arc<dyn Cache + Send + Sync>, // Use the Cache trait to allow for different cache implementations
}

impl Prefetcher {
    fn new(cache: Arc<dyn Cache + Send + Sync>) -> Self {
        Self {
            access_records: Arc::new(Mutex::new(HashMap::new())),
            cache,
        }
    }

    async fn record_access(&self, key: &str) {
        let mut access_records = self.access_records.lock().await;
        let now = Instant::now();

        let record = access_records.entry(key.to_owned()).or_insert_with(|| AccessRecord {
            last_access: now,
            frequency: Duration::from_secs(60), // Initialize with a high frequency to ensure it gets updated
        });

        record.frequency = now.duration_since(record.last_access);
        record.last_access = now;
    }

    async fn predict_and_prefetch(&self) {
        let access_records = self.access_records.lock().await;

        for (key, record) in access_records.iter() {
            if record.frequency < Duration::from_secs(5) {
                println!("Prefetching key: {}", key);
                let data = load_data_for_key(key).await;
                self.cache.set(key.clone(), data).await;
            }
        }
    }
}
