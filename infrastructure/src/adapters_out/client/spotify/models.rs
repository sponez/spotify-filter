use std::time::Instant;

pub(crate) struct CachedSnapshot {
    pub snapshot_id: String,
    pub cached_at: Instant,
}

