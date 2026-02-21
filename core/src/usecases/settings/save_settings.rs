use std::sync::Arc;

use crate::ports::{
    ports_in::settings::{
        models::SaveSettingsCommand,
        usecases::save_settings::SaveSettingsUseCase,
    },
    ports_out::repository::settings::{SettingsCache, SettingsStore},
};

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
    fn save_settings(&self, command: SaveSettingsCommand) {
        self.cache.store(&command.filter_action, &command.filter_target);

        let file = Arc::clone(&self.file);
        let action = command.filter_action.clone();
        let target = command.filter_target;
        std::thread::spawn(move || {
            file.save(&action, &target);
        });
    }
}
