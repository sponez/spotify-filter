use domain::ports::ports_out::repository::token::{RefreshTokenStore, TokenStoreError};
use tracing::{error, info};

pub struct KeyringRefreshTokenStore {
    service: String,
    user: String,
}

impl KeyringRefreshTokenStore {
    pub fn new(service: String, user: String) -> Self {
        Self { service, user }
    }

    fn entry(&self) -> Result<keyring::Entry, TokenStoreError> {
        keyring::Entry::new(&self.service, &self.user)
            .map_err(|e| TokenStoreError::LoadFailed(anyhow::anyhow!("{e}")))
    }
}

impl RefreshTokenStore for KeyringRefreshTokenStore {
    fn load(&self) -> Result<Option<String>, TokenStoreError> {
        info!("Loading refresh token from keyring");
        match self.entry()?.get_password() {
            Ok(pw) => Ok(Some(pw)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => {
                error!(error = %e, "Failed to load refresh token from keyring");
                Err(TokenStoreError::LoadFailed(anyhow::anyhow!("{e}")))
            }
        }
    }

    fn store(&self, refresh_token: &str) -> Result<(), TokenStoreError> {
        info!(len = refresh_token.len(), "Storing refresh token to keyring");
        self.entry()?
            .set_password(refresh_token)
            .map_err(|e| {
                error!(error = %e, "Failed to store refresh token to keyring");
                TokenStoreError::StoreFailed(anyhow::anyhow!("{e}"))
            })
    }

    fn delete(&self) -> Result<(), TokenStoreError> {
        info!("Deleting refresh token from keyring");
        match self.entry()?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => {
                error!(error = %e, "Failed to delete refresh token from keyring");
                Err(TokenStoreError::DeleteFailed(anyhow::anyhow!("{e}")))
            }
        }
    }
}
