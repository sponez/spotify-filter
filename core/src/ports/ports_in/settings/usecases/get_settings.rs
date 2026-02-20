use crate::ports::ports_in::settings::models::SettingsView;

pub trait GetSettingsUseCase {
    fn get_settings(&self) -> SettingsView;
}
