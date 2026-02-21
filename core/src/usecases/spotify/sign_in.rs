use std::sync::Arc;

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::sign_in::SignInUseCase,
        ports_out::notification::ErrorNotification,
    },
};

pub struct SignInInteractor {
    notifier: Arc<dyn ErrorNotification>,
}

impl SignInInteractor {
    pub fn new(notifier: Arc<dyn ErrorNotification>) -> Self {
        Self { notifier }
    }
}

impl SignInUseCase for SignInInteractor {
    fn sign_in(&self) -> AppResult<()> {
        self.notifier.notify("Sign in is not implemented yet");
        Ok(())
    }
}
