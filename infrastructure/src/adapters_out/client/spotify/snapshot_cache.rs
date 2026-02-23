use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::adapters_out::client::spotify::models::CachedSnapshot;

const SNAPSHOT_CACHE_TTL: Duration = Duration::from_secs(300);

pub(crate) struct SnapshotCache {
    inner: Mutex<HashMap<String, CachedSnapshot>>,
}

impl SnapshotCache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, playlist_id: &str) -> Option<String> {
        let mut cache = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let now = Instant::now();
        if let Some(entry) = cache.get(playlist_id) {
            if now.saturating_duration_since(entry.cached_at) < SNAPSHOT_CACHE_TTL {
                return Some(entry.snapshot_id.clone());
            }
        }
        cache.remove(playlist_id);
        None
    }

    pub fn put(&self, playlist_id: &str, snapshot_id: &str) {
        let mut cache = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        cache.insert(
            playlist_id.to_string(),
            CachedSnapshot {
                snapshot_id: snapshot_id.to_string(),
                cached_at: Instant::now(),
            },
        );
    }

    pub fn invalidate(&self, playlist_id: &str) {
        let mut cache = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        cache.remove(playlist_id);
    }
}

