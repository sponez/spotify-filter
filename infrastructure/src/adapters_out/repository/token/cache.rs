use std::sync::{Mutex, MutexGuard};

use domain::ports::ports_out::repository::token::TokenCache;

pub struct LocalTokenCache {
    inner: Mutex<Option<String>>,
}

impl LocalTokenCache {
    pub fn new() -> Self {
        Self { inner: Mutex::new(None) }
    }

    fn lock_or_reset(&self) -> MutexGuard<'_, Option<String>> {
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
        self.lock_or_reset().clone()
    }

    fn store(&self, access_token: &str) {
        let mut g = self.lock_or_reset();
        *g = Some(access_token.to_string());
    }

    fn clear(&self) {
        let mut g = self.lock_or_reset();
        *g = None;
    }
}
