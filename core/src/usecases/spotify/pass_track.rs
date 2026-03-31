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

        if track.is_local {
            return self.pass_local_track(&settings.pass_action);
        }

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

    fn pass_local_track(&self, action: &PassActionView) -> AppResult<()> {
        match action {
            PassActionView::None => {
                info!("Pass action for local track: skip only");
                self.notifier.notify("Local track is ignored");
            }
            PassActionView::AddToPlaylist | PassActionView::MoveToPlaylist => {
                info!("Pass action for local track is not supported by Spotify Web API");
                self.notifier.notify("Local track is ignored");
            }
        }
        self.api_client.skip_to_next()?;
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        errors::errors::AppResult,
        ports::{
            ports_in::settings::{
                models::{PassActionView, PassTargetView, SettingsView},
                usecases::get_settings::GetSettingsUseCase,
            },
            ports_out::{
                client::spotify_api::{
                    CurrentlyPlayingResponse, PlaylistSummary, SpotifyApiClient,
                },
                notification::ErrorNotification,
            },
        },
        usecases::spotify::pass_track::PassTrackInteractor,
    };

    #[derive(Default)]
    struct TestState {
        added_to_library: Vec<Vec<String>>,
        removed_from_library: Vec<Vec<String>>,
        added_to_playlist: Vec<(String, Vec<String>)>,
        removed_from_playlist: Vec<(String, Vec<String>)>,
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

        fn add_to_library(&self, uris: &[&str]) -> AppResult<()> {
            self.state
                .lock()
                .unwrap()
                .added_to_library
                .push(uris.iter().map(|uri| (*uri).to_string()).collect());
            Ok(())
        }

        fn remove_from_library(&self, uris: &[&str]) -> AppResult<()> {
            self.state
                .lock()
                .unwrap()
                .removed_from_library
                .push(uris.iter().map(|uri| (*uri).to_string()).collect());
            Ok(())
        }

        fn add_to_playlist(&self, playlist_id: &str, uris: &[&str]) -> AppResult<()> {
            self.state.lock().unwrap().added_to_playlist.push((
                playlist_id.to_string(),
                uris.iter().map(|uri| (*uri).to_string()).collect(),
            ));
            Ok(())
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

    struct TestSettings {
        settings: SettingsView,
    }

    impl GetSettingsUseCase for TestSettings {
        fn get_settings(&self) -> AppResult<SettingsView> {
            Ok(self.settings.clone())
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
    fn local_add_or_move_does_not_enqueue_mutations() {
        let state = Arc::new(Mutex::new(TestState::default()));
        let interactor = PassTrackInteractor::new(
            Arc::new(TestSpotifyApiClient {
                state: Arc::clone(&state),
            }),
            Arc::new(TestSettings {
                settings: SettingsView {
                    pass_action: PassActionView::MoveToPlaylist,
                    pass_target: PassTargetView::Playlist("target".to_string()),
                },
            }),
            Arc::new(TestNotifier {
                state: Arc::clone(&state),
            }),
        );

        interactor
            .pass_track(CurrentlyPlayingResponse {
                context_uri: Some("spotify:playlist:source".to_string()),
                track_uri: "spotify:local:artist:album:track:123".to_string(),
                is_local: true,
            })
            .unwrap();

        let state = state.lock().unwrap();
        assert!(state.added_to_library.is_empty());
        assert!(state.removed_from_library.is_empty());
        assert!(state.added_to_playlist.is_empty());
        assert!(state.removed_from_playlist.is_empty());
        assert_eq!(state.skipped, 1);
        assert_eq!(state.notifications, vec!["Local track is ignored".to_string()]);
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
