use crate::{errors::errors::AppResult, ports::ports_in::settings::models::SettingsView};

pub trait GetSettingsUseCase: Send + Sync {
    fn get_settings(&self) -> AppResult<SettingsView>;
}
