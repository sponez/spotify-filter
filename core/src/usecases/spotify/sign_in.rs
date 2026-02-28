use std::sync::Arc;
use tracing::{error, info, warn};

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::sign_in::SignInUseCase,
        ports_out::{
            auth::{auth_url_builder::AuthUrlBuilder, pkce::PkceGenerator},
            browser::BrowserLauncher,
            client::spotify_auth::SpotifyAuthClient,
            notification::ErrorNotification,
            repository::token::{RefreshTokenStore, TokenCache},
            server::callback_server::{CallbackServer, CallbackServerError},
        },
    },
};

pub struct SignInInteractor {
    callback_server: Box<dyn CallbackServer>,
    pkce_generator: Arc<dyn PkceGenerator>,
    auth_url_builder: Arc<dyn AuthUrlBuilder>,
    browser: Arc<dyn BrowserLauncher>,
    auth_client: Arc<dyn SpotifyAuthClient>,
    token_cache: Arc<dyn TokenCache>,
    refresh_token_store: Arc<dyn RefreshTokenStore>,
    notifier: Arc<dyn ErrorNotification>,
}

pub struct SignInDependencies {
    pub callback_server: Box<dyn CallbackServer>,
    pub pkce_generator: Arc<dyn PkceGenerator>,
    pub auth_url_builder: Arc<dyn AuthUrlBuilder>,
    pub browser: Arc<dyn BrowserLauncher>,
    pub auth_client: Arc<dyn SpotifyAuthClient>,
    pub token_cache: Arc<dyn TokenCache>,
    pub refresh_token_store: Arc<dyn RefreshTokenStore>,
    pub notifier: Arc<dyn ErrorNotification>,
}

impl SignInInteractor {
    pub fn new(deps: SignInDependencies) -> Self {
        Self {
            callback_server: deps.callback_server,
            pkce_generator: deps.pkce_generator,
            auth_url_builder: deps.auth_url_builder,
            browser: deps.browser,
            auth_client: deps.auth_client,
            token_cache: deps.token_cache,
            refresh_token_store: deps.refresh_token_store,
            notifier: deps.notifier,
        }
    }
}

impl SignInUseCase for SignInInteractor {
    fn sign_in(&self) -> AppResult<()> {
        info!("Sign-in started");
        let handle = self.callback_server.start().map_err(|e| {
            error!(error = %e, "Failed to start callback server");
            self.notifier.notify(&e.to_string());
            e
        })?;

        let pkce = self.pkce_generator.generate();
        let url = self
            .auth_url_builder
            .build_authorize_url(&pkce.challenge, &pkce.state);
        info!("Opening browser for Spotify authorization");

        self.browser.open_url(&url).map_err(|e| {
            error!(error = %e, "Failed to open browser");
            self.notifier.notify(&e.to_string());
            e
        })?;

        info!("Waiting for OAuth callback");
        let response = handle.wait_for_callback().map_err(|e| {
            error!(error = %e, "Failed while waiting for callback");
            self.notifier.notify(&e.to_string());
            e
        })?;

        if response.state != pkce.state {
            let err = CallbackServerError::StateMismatch;
            warn!("OAuth state mismatch");
            self.notifier.notify(&err.to_string());
            return Err(err.into());
        }

        info!("Exchanging authorization code for tokens");
        let tokens = self
            .auth_client
            .exchange_code(&response.code, &pkce.verifier)
            .map_err(|e| {
                error!(error = %e, "Token exchange failed");
                self.notifier.notify(&e.to_string());
                e
            })?;

        info!("Token exchange succeeded");

        self.token_cache
            .store(&tokens.access_token, tokens.expires_in);
        self.refresh_token_store
            .store(&tokens.refresh_token)
            .map_err(|e| {
                error!(error = %e, "Failed to persist refresh token");
                self.notifier.notify(&e.to_string());
                e
            })?;

        info!("Sign-in completed");
        Ok(())
    }
}
