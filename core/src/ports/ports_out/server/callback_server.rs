use thiserror::Error;

use crate::errors::errors::AppResult;

#[derive(Debug, Error)]
pub enum CallbackServerError {
    #[error("Failed to start callback server")]
    StartFailed(#[source] anyhow::Error),

    #[error("Failed to receive callback")]
    ReceiveFailed(#[source] anyhow::Error),

    #[error("Missing authorization code in callback")]
    MissingCode,
}

pub trait CallbackServer: Send + Sync {
    fn wait_for_callback(&self) -> AppResult<String>;
}
