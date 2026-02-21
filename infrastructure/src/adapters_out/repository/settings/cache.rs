use std::sync::{Arc, Mutex};

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
    inner: Arc<Mutex<Option<SettingsCacheDto>>>,
}

impl LocalSettingsCache {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(None)) }
    }
}

impl SettingsCache for LocalSettingsCache {
    fn load(&self) -> Option<(FilterActionView, FilterTargetView)> {
        self.inner.lock().unwrap().clone().map(cache_dto_to_view)
    }

    fn store(&self, action: &FilterActionView, target: &FilterTargetView) {
        *self.inner.lock().unwrap() = Some(view_to_cache_dto(action, target));
    }
}
