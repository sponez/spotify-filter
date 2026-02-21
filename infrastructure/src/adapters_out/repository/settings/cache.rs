use std::sync::{Mutex, MutexGuard};

use domain::ports::{
    ports_in::settings::models::{FilterActionView, FilterTargetView},
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
        Self { inner: Mutex::new(None) }
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
    fn load(&self) -> Option<(FilterActionView, FilterTargetView)> {
        let g = self.lock_or_reset();
        g.clone().map(cache_dto_to_view)
    }

    fn store(&self, action: &FilterActionView, target: &FilterTargetView) {
        let mut g = self.lock_or_reset();
        *g = Some(view_to_cache_dto(action, target));
    }
}
