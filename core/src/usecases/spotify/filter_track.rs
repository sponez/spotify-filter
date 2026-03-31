use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::{
    domain::{
        models::spotify_uri::{SpotifyUriType, SpotifyUserSubpath},
        uri_parser::parse_spotify_uri,
    },
    errors::errors::AppResult,
    ports::{
        ports_in::spotify::usecases::filter_track::FilterTrackUseCase,
        ports_out::{
            client::spotify_api::{CurrentlyPlayingResponse, SpotifyApiClient},
            notification::ErrorNotification,
        },
    },
};

pub struct FilterTrackInteractor {
    api_client: Arc<dyn SpotifyApiClient>,
    notifier: Arc<dyn ErrorNotification>,
}

impl FilterTrackInteractor {
    const POST_QUEUE_DELAY: Duration = Duration::from_secs(1);

    pub fn new(
        api_client: Arc<dyn SpotifyApiClient>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        Self {
            api_client,
            notifier,
        }
    }

    fn filter_track(&self, track: CurrentlyPlayingResponse) -> AppResult<()> {
        if track.is_local {
            return self.filter_local_track(track);
        }

        if let Some(context_uri_str) = track.context_uri {
            debug!(context_uri = %context_uri_str, track_uri = %track.track_uri, "Filtering track by context");
            let context_uri = parse_spotify_uri(&context_uri_str)?;

            if context_uri.uri_type == SpotifyUriType::Playlist {
                self.filter_playlist_track(&context_uri.id, &track.track_uri)?;
            }
            if (context_uri.uri_type == SpotifyUriType::User)
                && (context_uri.user_subpath == Some(SpotifyUserSubpath::Collection))
            {
                self.filter_user_collection_track(&track.track_uri)?;
            }
        }
        Ok(())
    }

    fn filter_local_track(&self, track: CurrentlyPlayingResponse) -> AppResult<()> {
        debug!(track_uri = %track.track_uri, "Ignoring local track");
        self.notifier.notify("Local track is ignored");
        self.api_client.skip_to_next()?;
        Ok(())
    }

    fn filter_playlist_track(&self, playlist_id: &str, track_uri: &str) -> AppResult<()> {
        info!(playlist_id, track_uri, "Filtering track from playlist");
        self.api_client
            .remove_from_playlist(playlist_id, &[track_uri])?;
        std::thread::sleep(Self::POST_QUEUE_DELAY);
        self.api_client.skip_to_next()?;
        Ok(())
    }

    fn filter_user_collection_track(&self, track_id: &str) -> AppResult<()> {
        info!(track_id, "Filtering track from liked songs");
        self.api_client.remove_from_library(&[track_id])?;
        std::thread::sleep(Self::POST_QUEUE_DELAY);
        self.api_client.skip_to_next()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        errors::errors::AppResult,
        ports::ports_out::{
            client::spotify_api::{CurrentlyPlayingResponse, PlaylistSummary, SpotifyApiClient},
            notification::ErrorNotification,
        },
        usecases::spotify::filter_track::FilterTrackInteractor,
    };

    #[derive(Default)]
    struct TestState {
        removed_from_playlist: Vec<(String, Vec<String>)>,
        removed_from_library: Vec<Vec<String>>,
        skipped: usize,
        notifications: Vec<String>,
    }

    struct TestSpotifyApiClient {
        state: Arc<Mutex<TestState>>,
    }

    impl SpotifyApiClient for TestSpotifyApiClient {
        fn get_currently_playing(&self) -> AppResult<Option<CurrentlyPlayingResponse>> {
            unreachable!()
        }

        fn get_my_playlists(&self) -> AppResult<Vec<PlaylistSummary>> {
            unreachable!()
        }

        fn add_to_library(&self, _uris: &[&str]) -> AppResult<()> {
            unreachable!()
        }

        fn remove_from_library(&self, uris: &[&str]) -> AppResult<()> {
            self.state
                .lock()
                .unwrap()
                .removed_from_library
                .push(uris.iter().map(|uri| (*uri).to_string()).collect());
            Ok(())
        }

        fn add_to_playlist(&self, _playlist_id: &str, _uris: &[&str]) -> AppResult<()> {
            unreachable!()
        }

        fn remove_from_playlist(&self, playlist_id: &str, uris: &[&str]) -> AppResult<()> {
            self.state.lock().unwrap().removed_from_playlist.push((
                playlist_id.to_string(),
                uris.iter().map(|uri| (*uri).to_string()).collect(),
            ));
            Ok(())
        }

        fn skip_to_next(&self) -> AppResult<()> {
            self.state.lock().unwrap().skipped += 1;
            Ok(())
        }
    }

    struct TestNotifier {
        state: Arc<Mutex<TestState>>,
    }

    impl ErrorNotification for TestNotifier {
        fn notify(&self, message: &str) {
            self.state
                .lock()
                .unwrap()
                .notifications
                .push(message.to_string());
        }
    }

    #[test]
    fn local_playlist_track_is_ignored_with_notification() {
        let state = Arc::new(Mutex::new(TestState::default()));
        let interactor = FilterTrackInteractor::new(
            Arc::new(TestSpotifyApiClient {
                state: Arc::clone(&state),
            }),
            Arc::new(TestNotifier {
                state: Arc::clone(&state),
            }),
        );

        interactor
            .filter_track(CurrentlyPlayingResponse {
                context_uri: Some("spotify:playlist:playlist123".to_string()),
                track_uri: "spotify:local:artist:album:track:123".to_string(),
                is_local: true,
            })
            .unwrap();

        let state = state.lock().unwrap();
        assert!(state.removed_from_playlist.is_empty());
        assert!(state.removed_from_library.is_empty());
        assert_eq!(state.skipped, 1);
        assert_eq!(
            state.notifications,
            vec!["Local track is ignored".to_string()]
        );
    }

    #[test]
    fn local_track_outside_playlist_skips_only_with_notification() {
        let state = Arc::new(Mutex::new(TestState::default()));
        let interactor = FilterTrackInteractor::new(
            Arc::new(TestSpotifyApiClient {
                state: Arc::clone(&state),
            }),
            Arc::new(TestNotifier {
                state: Arc::clone(&state),
            }),
        );

        interactor
            .filter_track(CurrentlyPlayingResponse {
                context_uri: None,
                track_uri: "spotify:local:artist:album:track:123".to_string(),
                is_local: true,
            })
            .unwrap();

        let state = state.lock().unwrap();
        assert!(state.removed_from_playlist.is_empty());
        assert!(state.removed_from_library.is_empty());
        assert_eq!(state.skipped, 1);
        assert_eq!(
            state.notifications,
            vec!["Local track is ignored".to_string()]
        );
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
