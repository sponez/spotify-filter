use crate::ports::ports_in::settings::models::SaveSettingsCommand;

pub trait SaveSettingsUseCase {
    fn save_settings(&self, command: SaveSettingsCommand);
}
