use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use domain::ports::ports_out::repository::token::TokenCache;

const REFRESH_THRESHOLD_SECS: u64 = 300; // 5 minutes

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

pub struct LocalTokenCache {
    inner: Mutex<Option<CachedToken>>,
}

impl LocalTokenCache {
    pub fn new() -> Self {
        Self { inner: Mutex::new(None) }
    }

    fn lock_or_reset(&self) -> MutexGuard<'_, Option<CachedToken>> {
        match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => {
                let mut g = poisoned.into_inner();
                *g = None;
                g
            }
        }
    }
}

impl TokenCache for LocalTokenCache {
    fn load(&self) -> Option<String> {
        self.lock_or_reset().as_ref().map(|t| t.access_token.clone())
    }

    fn store(&self, access_token: &str, expires_in_secs: u64) {
        let mut g = self.lock_or_reset();
        *g = Some(CachedToken {
            access_token: access_token.to_string(),
            expires_at: Instant::now() + Duration::from_secs(expires_in_secs),
        });
    }

    fn is_expiring_soon(&self) -> bool {
        let g = self.lock_or_reset();
        match g.as_ref() {
            None => true,
            Some(t) => {
                let remaining = t.expires_at.saturating_duration_since(Instant::now());
                remaining.as_secs() < REFRESH_THRESHOLD_SECS
            }
        }
    }

    fn clear(&self) {
        let mut g = self.lock_or_reset();
        *g = None;
    }
}
