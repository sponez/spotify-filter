use std::sync::Arc;
use tracing::{error, info};

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::sign_out::SignOutUseCase,
        ports_out::{
            notification::ErrorNotification,
            repository::token::{RefreshTokenStore, TokenCache},
        },
    },
};

pub struct SignOutInteractor {
    token_cache: Arc<dyn TokenCache>,
    refresh_token_store: Arc<dyn RefreshTokenStore>,
    notifier: Arc<dyn ErrorNotification>,
}

impl SignOutInteractor {
    pub fn new(
        token_cache: Arc<dyn TokenCache>,
        refresh_token_store: Arc<dyn RefreshTokenStore>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        Self {
            token_cache,
            refresh_token_store,
            notifier,
        }
    }
}

impl SignOutUseCase for SignOutInteractor {
    fn sign_out(&self) -> AppResult<()> {
        info!("Sign-out started");
        self.token_cache.clear();
        self.refresh_token_store.delete().map_err(|e| {
            error!(error = %e, "Failed to delete refresh token");
            self.notifier.notify(&e.to_string());
            e
        })?;
        info!("Sign-out completed");
        Ok(())
    }
}
