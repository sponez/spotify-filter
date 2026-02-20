use crate::ports::ports_in::spotify::usecases::pass_track::PassTrackUseCase;

pub struct PassTrackInteractor;

impl PassTrackInteractor {
    pub fn new() -> Self {
        Self
    }
}

impl PassTrackUseCase for PassTrackInteractor {
    fn pass_current_track(&self) {}
}
