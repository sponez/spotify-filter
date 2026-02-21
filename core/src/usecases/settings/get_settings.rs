use std::sync::Arc;

use crate::{errors::errors::AppResult, ports::{
    ports_in::settings::{
        models::SettingsView,
        usecases::get_settings::GetSettingsUseCase,
    },
    ports_out::repository::settings::{SettingsCache, SettingsStore},
}};

pub struct GetSettingsInteractor {
    cache: Arc<dyn SettingsCache>,
    file: Arc<dyn SettingsStore>,
}

impl GetSettingsInteractor {
    pub fn new(cache: Arc<dyn SettingsCache>, file: Arc<dyn SettingsStore>) -> Self {
        Self { cache, file }
    }
}

impl GetSettingsUseCase for GetSettingsInteractor {
    fn get_settings(&self) -> AppResult<SettingsView> {
        let current_cache = self.cache.load();
        let (filter_action, filter_target) = match current_cache {
            Some(pair) => pair,
            None => {
                let pair = self.file.load()?;
                self.cache.store(&pair.0, &pair.1);
                pair
            }
        };
        
        Ok(SettingsView { filter_action, filter_target })
    }
}
