use thiserror::Error;

use crate::ports::ports_out::repository::settings::SettingsStoreError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Settings(#[from] SettingsStoreError),

    #[error("Unexpected: {0}")]
    Unexpected(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Unexpected(e)
    }
}

pub type AppResult<T> = Result<T, AppError>;
