use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use std::sync::Arc;

use domain::ports::ports_out::{
    client::spotify_auth::SpotifyAuthClient,
    repository::token::{RefreshTokenStore, TokenCache},
};
use tracing::{debug, error, info, warn};

const REFRESH_THRESHOLD_SECS: u64 = 300; // 5 minutes

#[derive(Clone)]
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

struct RefreshDeps {
    auth_client: Arc<dyn SpotifyAuthClient>,
    refresh_token_store: Arc<dyn RefreshTokenStore>,
}

pub struct LocalTokenCache {
    inner: Mutex<Option<CachedToken>>,
    refresh_lock: Mutex<()>,
    refresh_deps: Option<RefreshDeps>,
}

impl LocalTokenCache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
            refresh_lock: Mutex::new(()),
            refresh_deps: None,
        }
    }

    pub fn with_auto_refresh(
        auth_client: Arc<dyn SpotifyAuthClient>,
        refresh_token_store: Arc<dyn RefreshTokenStore>,
    ) -> Self {
        Self {
            inner: Mutex::new(None),
            refresh_lock: Mutex::new(()),
            refresh_deps: Some(RefreshDeps {
                auth_client,
                refresh_token_store,
            }),
        }
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

    fn is_expiring_soon_token(token: &CachedToken) -> bool {
        token
            .expires_at
            .saturating_duration_since(Instant::now())
            .as_secs()
            < REFRESH_THRESHOLD_SECS
    }

    fn is_expired_token(token: &CachedToken) -> bool {
        Instant::now() >= token.expires_at
    }

    fn try_refresh(&self) -> bool {
        let Some(deps) = self.refresh_deps.as_ref() else {
            debug!("Auto-refresh is not configured for token cache");
            return false;
        };

        let _refresh_guard = match self.refresh_lock.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        let current = self.lock_or_reset().clone();
        let Some(current) = current else {
            return false;
        };
        if !Self::is_expiring_soon_token(&current) {
            return true;
        }

        info!("Access token is expiring soon, attempting auto-refresh");
        let refresh_token = match deps.refresh_token_store.load() {
            Ok(Some(token)) => token,
            Ok(None) => {
                warn!("Auto-refresh skipped: refresh token not found");
                return false;
            }
            Err(e) => {
                error!(error = %e, "Auto-refresh failed to load refresh token");
                return false;
            }
        };

        let tokens = match deps.auth_client.refresh_token(&refresh_token) {
            Ok(tokens) => tokens,
            Err(e) => {
                error!(error = %e, "Auto-refresh request failed");
                return false;
            }
        };

        {
            let mut g = self.lock_or_reset();
            *g = Some(CachedToken {
                access_token: tokens.access_token.clone(),
                expires_at: Instant::now() + Duration::from_secs(tokens.expires_in),
            });
        }

        if tokens.refresh_token != refresh_token
            && let Err(e) = deps.refresh_token_store.store(&tokens.refresh_token)
        {
            warn!(error = %e, "Failed to persist rotated refresh token after auto-refresh");
        }

        info!("Auto-refresh succeeded");
        true
    }
}

impl Default for LocalTokenCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenCache for LocalTokenCache {
    fn load(&self) -> Option<String> {
        debug!("Loading access token from in-memory cache");
        let cached = self.lock_or_reset().clone()?;
        if !Self::is_expiring_soon_token(&cached) {
            return Some(cached.access_token);
        }

        if self.try_refresh() {
            return self
                .lock_or_reset()
                .as_ref()
                .map(|t| t.access_token.clone());
        }

        if Self::is_expired_token(&cached) {
            warn!("Access token is expired and auto-refresh failed, clearing cache entry");
            let mut g = self.lock_or_reset();
            *g = None;
            return None;
        }

        warn!("Auto-refresh failed, returning near-expiry access token");
        Some(cached.access_token)
    }

    fn store(&self, access_token: &str, expires_in_secs: u64) {
        debug!(
            expires_in_secs,
            token_len = access_token.len(),
            "Storing access token in cache"
        );
        let mut g = self.lock_or_reset();
        *g = Some(CachedToken {
            access_token: access_token.to_string(),
            expires_at: Instant::now() + Duration::from_secs(expires_in_secs),
        });
    }

    fn is_expiring_soon(&self) -> bool {
        let g = self.lock_or_reset();
        match g.as_ref() {
            None => {
                debug!("Access token is missing from cache");
                true
            }
            Some(t) => {
                let remaining = t.expires_at.saturating_duration_since(Instant::now());
                debug!(
                    remaining_secs = remaining.as_secs(),
                    "Checked access token expiration"
                );
                remaining.as_secs() < REFRESH_THRESHOLD_SECS
            }
        }
    }

    fn clear(&self) {
        warn!("Clearing access token cache");
        let mut g = self.lock_or_reset();
        *g = None;
    }
}
