use crate::ports::ports_in::spotify::usecases::filter_track::FilterTrackUseCase;

pub struct FilterTrackInteractor;

impl FilterTrackInteractor {
    pub fn new() -> Self {
        Self
    }
}

impl FilterTrackUseCase for FilterTrackInteractor {
    fn filter_current_track(&self) {}
}
