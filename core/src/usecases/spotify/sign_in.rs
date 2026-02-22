use std::sync::Arc;

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::sign_in::SignInUseCase,
        ports_out::{
            notification::ErrorNotification,
            server::callback_server::CallbackServer,
        },
    },
};

pub struct SignInInteractor {
    callback_server: Box<dyn CallbackServer>,
    notifier: Arc<dyn ErrorNotification>,
}

impl SignInInteractor {
    pub fn new(
        callback_server: Box<dyn CallbackServer>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        Self { callback_server, notifier }
    }
}

impl SignInUseCase for SignInInteractor {
    fn sign_in(&self) -> AppResult<()> {
        let code = self.callback_server.wait_for_callback().map_err(|e| {
            self.notifier.notify(&e.to_string());
            e
        })?;
        println!("Received auth code: {code}");
        Ok(())
    }
}
