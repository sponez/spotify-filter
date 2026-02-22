use std::sync::Arc;

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::filter_track::FilterTrackUseCase,
        ports_out::{
            client::spotify_api::SpotifyApiClient,
            notification::ErrorNotification,
        },
    },
};

pub struct FilterTrackInteractor {
    api_client: Arc<dyn SpotifyApiClient>,
    notifier: Arc<dyn ErrorNotification>,
}

impl FilterTrackInteractor {
    pub fn new(
        api_client: Arc<dyn SpotifyApiClient>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        Self { api_client, notifier }
    }
}

impl FilterTrackUseCase for FilterTrackInteractor {
    fn filter_current_track(&self) -> AppResult<()> {
        match self.api_client.get_currently_playing() {
            Ok(Some(track)) => {
                println!(
                    "Currently playing: {} - {}",
                    track.artist_names.join(", "),
                    track.track_name,
                );
            }
            Ok(None) => {
                println!("Nothing is currently playing");
            }
            Err(e) => {
                self.notifier.notify(&e.to_string());
                return Err(e);
            }
        }
        Ok(())
    }
}
