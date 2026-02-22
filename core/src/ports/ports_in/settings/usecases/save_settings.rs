use crate::{
    errors::errors::AppResult,
    ports::ports_in::settings::models::SaveSettingsCommand
};

pub trait SaveSettingsUseCase: Send + Sync {
    fn save_settings(&self, command: SaveSettingsCommand) -> AppResult<()>;
}
