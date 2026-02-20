use std::sync::Arc;

use crate::ports::ports_in::settings::usecases::{
    get_settings::GetSettingsUseCase,
    save_settings::SaveSettingsUseCase,
};

pub struct SettingsFacade {
    pub get: Arc<dyn GetSettingsUseCase>,
    pub save: Arc<dyn SaveSettingsUseCase>,
}

impl SettingsFacade {
    pub fn new(
        get: Arc<dyn GetSettingsUseCase>,
        save: Arc<dyn SaveSettingsUseCase>,
    ) -> Self {
        Self { get, save }
    }
}
