use std::sync::{Mutex, MutexGuard};
use tracing::debug;

use domain::ports::{
    ports_in::settings::models::{PassActionView, PassTargetView},
    ports_out::repository::settings::SettingsCache,
};

use crate::adapters_out::repository::settings::{
    dto::settings_dto::SettingsCacheDto,
    mapper::settings_mapper::{cache_dto_to_view, view_to_cache_dto},
};

/// In-memory cache — starts cold (`None`), populated on first load.
pub struct LocalSettingsCache {
    inner: Mutex<Option<SettingsCacheDto>>,
}

impl LocalSettingsCache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    fn lock_or_reset(&self) -> MutexGuard<'_, Option<SettingsCacheDto>> {
        match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => {
                // A thread panicked while holding the lock.
                // Reset cache to a safe empty state.
                let mut g = poisoned.into_inner();
                *g = None;
                g
            }
        }
    }
}

impl SettingsCache for LocalSettingsCache {
    fn load(&self) -> Option<(PassActionView, PassTargetView)> {
        debug!("Loading settings from in-memory cache");
        let g = self.lock_or_reset();
        g.clone().map(cache_dto_to_view)
    }

    fn store(&self, action: &PassActionView, target: &PassTargetView) {
        debug!("Storing settings in in-memory cache");
        let mut g = self.lock_or_reset();
        *g = Some(view_to_cache_dto(action, target));
    }
}
