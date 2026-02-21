use std::sync::Arc;

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::pass_track::PassTrackUseCase,
        ports_out::notification::ErrorNotification,
    },
};

pub struct PassTrackInteractor {
    notifier: Arc<dyn ErrorNotification>,
}

impl PassTrackInteractor {
    pub fn new(notifier: Arc<dyn ErrorNotification>) -> Self {
        Self { notifier }
    }
}

impl PassTrackUseCase for PassTrackInteractor {
    fn pass_current_track(&self) -> AppResult<()> {
        self.notifier.notify("Pass track is not implemented yet");
        Ok(())
    }
}
