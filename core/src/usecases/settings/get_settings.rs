use std::sync::Arc;

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::settings::{
            models::SettingsView,
            usecases::get_settings::GetSettingsUseCase,
        },
        ports_out::{
            notification::ErrorNotification,
            repository::settings::{SettingsCache, SettingsStore},
        },
    },
};

pub struct GetSettingsInteractor {
    cache: Arc<dyn SettingsCache>,
    file: Arc<dyn SettingsStore>,
    notifier: Arc<dyn ErrorNotification>,
}

impl GetSettingsInteractor {
    pub fn new(
        cache: Arc<dyn SettingsCache>,
        file: Arc<dyn SettingsStore>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        Self { cache, file, notifier }
    }
}

impl GetSettingsUseCase for GetSettingsInteractor {
    fn get_settings(&self) -> AppResult<SettingsView> {
        let current_cache = self.cache.load();
        let (filter_action, filter_target) = match current_cache {
            Some(pair) => pair,
            None => {
                let pair = self.file.load().map_err(|e| {
                    self.notifier.notify(&e.to_string());
                    e
                })?;
                self.cache.store(&pair.0, &pair.1);
                pair
            }
        };

        Ok(SettingsView { pass_action: filter_action, pass_target: filter_target })
    }
}
