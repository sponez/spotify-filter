use std::sync::Arc;

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::filter_track::FilterTrackUseCase,
        ports_out::notification::ErrorNotification,
    },
};

pub struct FilterTrackInteractor {
    notifier: Arc<dyn ErrorNotification>,
}

impl FilterTrackInteractor {
    pub fn new(notifier: Arc<dyn ErrorNotification>) -> Self {
        Self { notifier }
    }
}

impl FilterTrackUseCase for FilterTrackInteractor {
    fn filter_current_track(&self) -> AppResult<()> {
        self.notifier.notify("Filter track is not implemented yet");
        Ok(())
    }
}
