use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenStoreError {
    #[error("Failed to store refresh token")]
    StoreFailed(#[source] anyhow::Error),

    #[error("Failed to load refresh token")]
    LoadFailed(#[source] anyhow::Error),

    #[error("Failed to delete refresh token")]
    DeleteFailed(#[source] anyhow::Error),
}

/// In-memory cache for the current access token.
pub trait TokenCache: Send + Sync {
    fn load(&self) -> Option<String>;
    fn store(&self, access_token: &str);
    fn clear(&self);
}

/// Durable storage for the refresh token (e.g. OS credential manager).
pub trait RefreshTokenStore: Send + Sync {
    fn load(&self) -> Result<Option<String>, TokenStoreError>;
    fn store(&self, refresh_token: &str) -> Result<(), TokenStoreError>;
    fn delete(&self) -> Result<(), TokenStoreError>;
}
