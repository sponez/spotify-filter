use std::sync::Arc;
use tracing::{error, info, warn};

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::try_sign_in::TrySignInUseCase,
        ports_out::{
            client::spotify_auth::SpotifyAuthClient,
            repository::token::{RefreshTokenStore, TokenCache},
        },
    },
};

pub struct TrySignInInteractor {
    auth_client: Arc<dyn SpotifyAuthClient>,
    token_cache: Arc<dyn TokenCache>,
    refresh_token_store: Arc<dyn RefreshTokenStore>,
}

impl TrySignInInteractor {
    pub fn new(
        auth_client: Arc<dyn SpotifyAuthClient>,
        token_cache: Arc<dyn TokenCache>,
        refresh_token_store: Arc<dyn RefreshTokenStore>,
    ) -> Self {
        Self {
            auth_client,
            token_cache,
            refresh_token_store,
        }
    }
}

impl TrySignInUseCase for TrySignInInteractor {
    fn try_sign_in(&self) -> AppResult<bool> {
        info!("Attempting silent sign-in");
        let Some(refresh_token) = self.refresh_token_store.load()? else {
            warn!("No refresh token found for silent sign-in");
            return Ok(false);
        };

        let tokens = self
            .auth_client
            .refresh_token(&refresh_token)
            .map_err(|e| {
                error!(error = %e, "Silent sign-in token refresh failed");
                e
            })?;
        self.token_cache
            .store(&tokens.access_token, tokens.expires_in);
        if tokens.refresh_token != refresh_token {
            self.refresh_token_store.store(&tokens.refresh_token)?;
        }
        info!("Silent sign-in succeeded");
        Ok(true)
    }
}
