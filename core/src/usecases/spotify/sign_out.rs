use std::sync::Arc;

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::sign_out::SignOutUseCase,
        ports_out::notification::ErrorNotification,
    },
};

pub struct SignOutInteractor {
    notifier: Arc<dyn ErrorNotification>,
}

impl SignOutInteractor {
    pub fn new(notifier: Arc<dyn ErrorNotification>) -> Self {
        Self { notifier }
    }
}

impl SignOutUseCase for SignOutInteractor {
    fn sign_out(&self) -> AppResult<()> {
        self.notifier.notify("Sign out is not implemented yet");
        Ok(())
    }
}
