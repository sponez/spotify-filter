use std::sync::Arc;

use crate::{errors::errors::AppResult, ports::{
    ports_in::settings::{
        models::SaveSettingsCommand,
        usecases::save_settings::SaveSettingsUseCase,
    },
    ports_out::repository::settings::{SettingsCache, SettingsStore},
}};

pub struct SaveSettingsInteractor {
    cache: Arc<dyn SettingsCache>,
    file: Arc<dyn SettingsStore>,
}

impl SaveSettingsInteractor {
    pub fn new(cache: Arc<dyn SettingsCache>, file: Arc<dyn SettingsStore>) -> Self {
        Self { cache, file }
    }
}

impl SaveSettingsUseCase for SaveSettingsInteractor {
    fn save_settings(&self, command: SaveSettingsCommand) -> AppResult<()> {
        let action = command.filter_action.clone();
        let target = command.filter_target.clone();

        self.file.save(&action, &target)?;
        self.cache.store(&action, &target);
        
        Ok(())
    }
}
