use domain::ports::ports_out::repository::token::{RefreshTokenStore, TokenStoreError};

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
        match self.entry()?.get_password() {
            Ok(pw) => Ok(Some(pw)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(TokenStoreError::LoadFailed(anyhow::anyhow!("{e}"))),
        }
    }

    fn store(&self, refresh_token: &str) -> Result<(), TokenStoreError> {
        self.entry()?
            .set_password(refresh_token)
            .map_err(|e| TokenStoreError::StoreFailed(anyhow::anyhow!("{e}")))
    }

    fn delete(&self) -> Result<(), TokenStoreError> {
        match self.entry()?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(TokenStoreError::DeleteFailed(anyhow::anyhow!("{e}"))),
        }
    }
}
