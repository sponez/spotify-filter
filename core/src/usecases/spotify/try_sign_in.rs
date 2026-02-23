use std::sync::Arc;

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
        Self { auth_client, token_cache, refresh_token_store }
    }
}

impl TrySignInUseCase for TrySignInInteractor {
    fn try_sign_in(&self) -> AppResult<bool> {
        let Some(refresh_token) = self.refresh_token_store.load()? else {
            return Ok(false);
        };

        let tokens = self.auth_client.refresh_token(&refresh_token)?;
        self.token_cache.store(&tokens.access_token, tokens.expires_in);
        if tokens.refresh_token != refresh_token {
            self.refresh_token_store.store(&tokens.refresh_token)?;
        }
        Ok(true)
    }
}
