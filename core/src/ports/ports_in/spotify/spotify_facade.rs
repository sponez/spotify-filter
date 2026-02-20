use std::sync::Arc;

use crate::ports::ports_in::spotify::usecases::{
    filter_track::FilterTrackUseCase,
    pass_track::PassTrackUseCase,
    sign_in::SignInUseCase,
    sign_out::SignOutUseCase,
};

pub struct SpotifyFacade {
    pub sign_in: Arc<dyn SignInUseCase>,
    pub sign_out: Arc<dyn SignOutUseCase>,
    pub pass_track: Arc<dyn PassTrackUseCase>,
    pub filter_track: Arc<dyn FilterTrackUseCase>,
}

impl SpotifyFacade {
    pub fn new(
        sign_in: Arc<dyn SignInUseCase>,
        sign_out: Arc<dyn SignOutUseCase>,
        pass_track: Arc<dyn PassTrackUseCase>,
        filter_track: Arc<dyn FilterTrackUseCase>,
    ) -> Self {
        Self {
            sign_in,
            sign_out,
            pass_track,
            filter_track,
        }
    }
}
