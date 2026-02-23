use std::sync::Arc;
use tracing::{debug, error, info};

use crate::{
    domain::{models::spotify_uri::{SpotifyUriType, SpotifyUserSubpath}, uri_parser::parse_spotify_uri}, errors::errors::AppResult, ports::{
        ports_in::spotify::usecases::filter_track::FilterTrackUseCase,
        ports_out::{
            client::spotify_api::{CurrentlyPlayingResponse, SpotifyApiClient},
            notification::ErrorNotification,
        },
    }
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

    fn filter_track(&self, track: CurrentlyPlayingResponse) -> AppResult<()> {
        if let Some(context_uri_str) = track.context_uri {
            debug!(context_uri = %context_uri_str, track_uri = %track.track_uri, "Filtering track by context");
            let context_uri = parse_spotify_uri(&context_uri_str)?;

            if context_uri.uri_type == SpotifyUriType::Playlist {
                self.filter_playlist_track(&context_uri.id, &track.track_uri)?;
            }
            if (context_uri.uri_type == SpotifyUriType::User) &&
                (context_uri.user_subpath == Some(SpotifyUserSubpath::Collection)) {
                self.filter_user_collection_track(&track.track_uri)?;
            }
        }
        Ok(())
    }

    fn filter_playlist_track(&self, playlist_id: &str, track_uri: &str) -> AppResult<()> {
        info!(playlist_id, track_uri, "Filtering track from playlist");
        let playlist_snapshot = self.api_client.get_playlist_snapshot(playlist_id)?;
        self.api_client.remove_from_playlist(playlist_id, &[track_uri], &playlist_snapshot.snapshot_id)?;
        self.api_client.skip_to_next()?;
        Ok(())
    }

    fn filter_user_collection_track(&self, track_id: &str) -> AppResult<()> {
        info!(track_id, "Filtering track from liked songs");
        self.api_client.remove_from_library(&[track_id])?;
        self.api_client.skip_to_next()?;
        Ok(())
    }
}

impl FilterTrackUseCase for FilterTrackInteractor {
    fn filter_current_track(&self) -> AppResult<()> {
        info!("Filter current track requested");
        match self.api_client.get_currently_playing() {
            Ok(Some(track)) => {
                self.filter_track(track).map_err(|e| {
                    error!(error = %e, "Failed to filter current track");
                    self.notifier.notify(&e.to_string());
                    e
                })?;
            }
            Ok(None) => {
                debug!("Nothing is currently playing");
            }
            Err(e) => {
                error!(error = %e, "Failed to read currently playing track");
                self.notifier.notify(&e.to_string());
                return Err(e);
            }
        }
        info!("Filter current track completed");
        Ok(())
    }
}
