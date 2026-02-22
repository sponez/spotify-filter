use std::sync::Arc;

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::sign_in::SignInUseCase,
        ports_out::{
            auth::{
                auth_url_builder::AuthUrlBuilder,
                pkce::PkceGenerator,
            },
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

impl SignInInteractor {
    pub fn new(
        callback_server: Box<dyn CallbackServer>,
        pkce_generator: Arc<dyn PkceGenerator>,
        auth_url_builder: Arc<dyn AuthUrlBuilder>,
        browser: Arc<dyn BrowserLauncher>,
        auth_client: Arc<dyn SpotifyAuthClient>,
        token_cache: Arc<dyn TokenCache>,
        refresh_token_store: Arc<dyn RefreshTokenStore>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        Self {
            callback_server,
            pkce_generator,
            auth_url_builder,
            browser,
            auth_client,
            token_cache,
            refresh_token_store,
            notifier,
        }
    }
}

impl SignInUseCase for SignInInteractor {
    fn sign_in(&self) -> AppResult<()> {
        let handle = self.callback_server.start().map_err(|e| {
            self.notifier.notify(&e.to_string());
            e
        })?;

        let pkce = self.pkce_generator.generate();
        let url = self.auth_url_builder.build_authorize_url(&pkce.challenge, &pkce.state);

        self.browser.open_url(&url).map_err(|e| {
            self.notifier.notify(&e.to_string());
            e
        })?;

        let response = handle.wait_for_callback().map_err(|e| {
            self.notifier.notify(&e.to_string());
            e
        })?;

        if response.state != pkce.state {
            let err = CallbackServerError::StateMismatch;
            self.notifier.notify(&err.to_string());
            return Err(err.into());
        }

        let tokens = self.auth_client.exchange_code(&response.code, &pkce.verifier).map_err(|e| {
            self.notifier.notify(&e.to_string());
            e
        })?;

        self.token_cache.store(&tokens.access_token);
        self.refresh_token_store.store(&tokens.refresh_token).map_err(|e| {
            self.notifier.notify(&e.to_string());
            e
        })?;

        Ok(())
    }
}
