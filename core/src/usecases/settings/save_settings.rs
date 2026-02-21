use std::sync::Arc;

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::settings::{
            models::SaveSettingsCommand,
            usecases::save_settings::SaveSettingsUseCase,
        },
        ports_out::{
            notification::ErrorNotification,
            repository::settings::{SettingsCache, SettingsStore},
        },
    },
};

pub struct SaveSettingsInteractor {
    cache: Arc<dyn SettingsCache>,
    file: Arc<dyn SettingsStore>,
    notifier: Arc<dyn ErrorNotification>,
}

impl SaveSettingsInteractor {
    pub fn new(
        cache: Arc<dyn SettingsCache>,
        file: Arc<dyn SettingsStore>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        Self { cache, file, notifier }
    }
}

impl SaveSettingsUseCase for SaveSettingsInteractor {
    fn save_settings(&self, command: SaveSettingsCommand) -> AppResult<()> {
        let action = command.filter_action.clone();
        let target = command.filter_target.clone();

        self.file.save(&action, &target).map_err(|e| {
            self.notifier.notify(&e.to_string());
            e
        })?;
        self.cache.store(&action, &target);

        Ok(())
    }
}
