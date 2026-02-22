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

    #[error("State mismatch in callback (possible CSRF attack)")]
    StateMismatch,
}

pub struct CallbackResponse {
    pub code: String,
    pub state: String,
}

pub trait CallbackServer: Send + Sync {
    fn start(&self) -> AppResult<Box<dyn CallbackHandle>>;
}

pub trait CallbackHandle: Send {
    fn wait_for_callback(&self) -> AppResult<CallbackResponse>;
}
