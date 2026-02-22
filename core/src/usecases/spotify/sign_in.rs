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
            notification::ErrorNotification,
            server::callback_server::{CallbackServer, CallbackServerError},
        },
    },
};

pub struct SignInInteractor {
    callback_server: Box<dyn CallbackServer>,
    pkce_generator: Arc<dyn PkceGenerator>,
    auth_url_builder: Arc<dyn AuthUrlBuilder>,
    browser: Arc<dyn BrowserLauncher>,
    notifier: Arc<dyn ErrorNotification>,
}

impl SignInInteractor {
    pub fn new(
        callback_server: Box<dyn CallbackServer>,
        pkce_generator: Arc<dyn PkceGenerator>,
        auth_url_builder: Arc<dyn AuthUrlBuilder>,
        browser: Arc<dyn BrowserLauncher>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        Self { callback_server, pkce_generator, auth_url_builder, browser, notifier }
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

        println!("Received auth code: {}, verifier: {}", response.code, pkce.verifier);
        Ok(())
    }
}
