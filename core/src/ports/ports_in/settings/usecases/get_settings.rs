use crate::{
    errors::errors::AppResult,
    ports::ports_in::settings::models::SettingsView
};

pub trait GetSettingsUseCase {
    fn get_settings(&self) -> AppResult<SettingsView>;
}
