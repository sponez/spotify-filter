use std::sync::Arc;

use crate::ports::{
    ports_in::settings::{
        models::SettingsView,
        usecases::get_settings::GetSettingsUseCase,
    },
    ports_out::settings::{SettingsCachePort, SettingsFilePort},
};

pub struct GetSettingsInteractor {
    cache: Arc<dyn SettingsCachePort>,
    file: Arc<dyn SettingsFilePort>,
}

impl GetSettingsInteractor {
    pub fn new(cache: Arc<dyn SettingsCachePort>, file: Arc<dyn SettingsFilePort>) -> Self {
        Self { cache, file }
    }
}

impl GetSettingsUseCase for GetSettingsInteractor {
    fn get_settings(&self) -> SettingsView {
        let (filter_action, filter_target) = match self.cache.load() {
            Some(pair) => pair,
            None => {
                let pair = self.file.load();
                self.cache.store(&pair.0, &pair.1);
                pair
            }
        };
        SettingsView { filter_action, filter_target }
    }
}
