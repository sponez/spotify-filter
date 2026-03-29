use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::{
    domain::{
        models::spotify_uri::{SpotifyUri, SpotifyUriType},
        uri_parser::parse_spotify_uri,
    },
    errors::errors::AppResult,
    ports::{
        ports_in::{
            settings::{
                models::{PassActionView, PassTargetView},
                usecases::get_settings::GetSettingsUseCase,
            },
            spotify::usecases::pass_track::PassTrackUseCase,
        },
        ports_out::{
            client::spotify_api::{CurrentlyPlayingResponse, SpotifyApiClient},
            notification::ErrorNotification,
        },
    },
};

pub struct PassTrackInteractor {
    api_client: Arc<dyn SpotifyApiClient>,
    settings_provider: Arc<dyn GetSettingsUseCase>,
    notifier: Arc<dyn ErrorNotification>,
}

impl PassTrackInteractor {
    const POST_QUEUE_DELAY: Duration = Duration::from_secs(1);

    pub fn new(
        api_client: Arc<dyn SpotifyApiClient>,
        settings_provider: Arc<dyn GetSettingsUseCase>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        Self {
            api_client,
            settings_provider,
            notifier,
        }
    }

    fn pass_track(&self, track: CurrentlyPlayingResponse) -> AppResult<()> {
        info!(track_uri = %track.track_uri, "Pass current track requested");
        let settings = self.settings_provider.get_settings()?;

        if let Some(context_uri_str) = track.context_uri {
            let context_uri = parse_spotify_uri(&context_uri_str)?;
            debug!(context_uri = %context_uri_str, "Resolved playback context");

            match settings.pass_action {
                PassActionView::None => {
                    info!("Pass action: skip only");
                    self.api_client.skip_to_next()?;
                }
                PassActionView::AddToPlaylist => {
                    info!("Pass action: add to target");
                    self.add_to_playlist(&context_uri, &settings.pass_target, &track.track_uri)?;
                }
                PassActionView::MoveToPlaylist => {
                    info!("Pass action: move to target");
                    self.move_to_playlist(&context_uri, &settings.pass_target, &track.track_uri)?;
                }
            }
        }
        Ok(())
    }

    fn add_to_playlist(
        &self,
        context_uri: &SpotifyUri,
        pass_target: &PassTargetView,
        track_uri: &str,
    ) -> AppResult<()> {
        self.add_to_target(context_uri, pass_target, track_uri)?;
        std::thread::sleep(Self::POST_QUEUE_DELAY);
        self.api_client.skip_to_next()?;
        Ok(())
    }

    fn move_to_playlist(
        &self,
        context_uri: &SpotifyUri,
        pass_target: &PassTargetView,
        track_uri: &str,
    ) -> AppResult<()> {
        self.add_to_target(context_uri, pass_target, track_uri)?;
        self.remove_from_source(context_uri, pass_target, track_uri)?;
        std::thread::sleep(Self::POST_QUEUE_DELAY);
        self.api_client.skip_to_next()?;
        Ok(())
    }

    fn add_to_target(
        &self,
        context_uri: &SpotifyUri,
        pass_target: &PassTargetView,
        track_uri: &str,
    ) -> AppResult<()> {
        match pass_target {
            PassTargetView::LikedSongs => {
                if !context_uri.is_collection() {
                    self.api_client.add_to_library(&[track_uri])?;
                }
            }
            PassTargetView::Playlist(playlist_id) => {
                if context_uri.uri_type != SpotifyUriType::Playlist
                    || context_uri.id != *playlist_id
                {
                    self.api_client.add_to_playlist(playlist_id, &[track_uri])?;
                }
            }
        }
        Ok(())
    }

    fn remove_from_source(
        &self,
        context_uri: &SpotifyUri,
        pass_target: &PassTargetView,
        track_uri: &str,
    ) -> AppResult<()> {
        match context_uri.uri_type {
            SpotifyUriType::Playlist => {
                if *pass_target != PassTargetView::Playlist(context_uri.id.clone()) {
                    self.api_client
                        .remove_from_playlist(&context_uri.id, &[track_uri])?;
                }
            }
            SpotifyUriType::User if context_uri.is_collection() => {
                if *pass_target != PassTargetView::LikedSongs {
                    self.api_client.remove_from_library(&[track_uri])?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl PassTrackUseCase for PassTrackInteractor {
    fn pass_current_track(&self) -> AppResult<()> {
        info!("Pass track requested");
        match self.api_client.get_currently_playing() {
            Ok(Some(track)) => {
                self.pass_track(track).map_err(|e| {
                    error!(error = %e, "Failed to pass current track");
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
        info!("Pass track completed");
        Ok(())
    }
}
