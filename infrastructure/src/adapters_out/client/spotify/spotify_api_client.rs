use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use domain::{
    errors::errors::AppResult,
    ports::ports_out::{
        client::spotify_api::{CurrentlyPlayingResponse, PlaylistSummary, SpotifyApiClient},
        notification::ErrorNotification,
        repository::token::TokenCache,
    },
};
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, error, info};

use crate::adapters_out::client::spotify::{
    action::SpotifyApiAction,
    playlist_sync_scheduler::{
        PLAYLIST_API_INTERVAL, PlaylistSyncScheduler, QueueMap, QueueTarget,
    },
    request_scheduler::{RequestScheduler, ScheduleMode},
};

const PLAYBACK_API_INTERVAL: Duration = Duration::from_secs(10);

pub struct UreqSpotifyApiClient {
    base_url: String,
    paths: HashMap<SpotifyApiAction, String>,
    token_cache: Arc<dyn TokenCache>,
    playlist_scheduler: Arc<RequestScheduler>,
    playback_scheduler: Arc<RequestScheduler>,
    add_queue: Arc<Mutex<QueueMap>>,
    remove_queue: Arc<Mutex<QueueMap>>,
    playlist_sync_scheduler: PlaylistSyncScheduler,
}

impl UreqSpotifyApiClient {
    pub fn new(
        base_url: String,
        paths: HashMap<SpotifyApiAction, String>,
        token_cache: Arc<dyn TokenCache>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        let playlist_scheduler = Arc::new(RequestScheduler::new(PLAYLIST_API_INTERVAL));
        let playback_scheduler = Arc::new(RequestScheduler::new(PLAYBACK_API_INTERVAL));
        let add_queue = Arc::new(Mutex::new(HashMap::new()));
        let remove_queue = Arc::new(Mutex::new(HashMap::new()));
        let playlist_sync_scheduler = PlaylistSyncScheduler::start(
            base_url.clone(),
            paths.clone(),
            Arc::clone(&token_cache),
            Arc::clone(&notifier),
            Arc::clone(&playlist_scheduler),
            Arc::clone(&add_queue),
            Arc::clone(&remove_queue),
        );

        Self {
            base_url,
            paths,
            token_cache,
            playlist_scheduler,
            playback_scheduler,
            add_queue,
            remove_queue,
            playlist_sync_scheduler,
        }
    }

    fn token(&self) -> AppResult<String> {
        Ok(self
            .token_cache
            .load()
            .ok_or_else(|| anyhow::anyhow!("No access token available"))?)
    }

    fn url(&self, action: SpotifyApiAction) -> AppResult<String> {
        let path = self
            .paths
            .get(&action)
            .ok_or_else(|| anyhow::anyhow!("No path configured for action '{action:?}'"))?;
        Ok(format!("{}{}", self.base_url, path))
    }

    fn schedule<T, F>(&self, op_name: &str, op: F) -> AppResult<T>
    where
        F: FnMut() -> Result<ureq::http::Response<ureq::Body>, ureq::Error>,
        T: serde::de::DeserializeOwned,
    {
        let mut response = self
            .playlist_scheduler
            .run(op_name, ScheduleMode::FailFast, op)?;
        response
            .body_mut()
            .read_json()
            .map_err(|e| anyhow::anyhow!("{e}").into())
    }

    fn schedule_playback<F>(&self, op_name: &str, op: F) -> AppResult<()>
    where
        F: FnMut() -> Result<ureq::http::Response<ureq::Body>, ureq::Error>,
    {
        self.playback_scheduler
            .run(op_name, ScheduleMode::FailFast, op)
            .map(|_| ())
    }

    fn schedule_playlist_wait<T, F>(&self, op_name: &str, op: F) -> AppResult<T>
    where
        F: FnMut() -> Result<ureq::http::Response<ureq::Body>, ureq::Error>,
        T: serde::de::DeserializeOwned,
    {
        let mut response = self
            .playlist_scheduler
            .run(op_name, ScheduleMode::Wait, op)?;
        response
            .body_mut()
            .read_json()
            .map_err(|e| anyhow::anyhow!("{e}").into())
    }

    fn schedule_playlist_wait_empty<F>(&self, op_name: &str, op: F) -> AppResult<()>
    where
        F: FnMut() -> Result<ureq::http::Response<ureq::Body>, ureq::Error>,
    {
        self.playlist_scheduler
            .run(op_name, ScheduleMode::Wait, op)
            .map(|_| ())
    }

    pub fn shutdown(&self) {
        self.playlist_sync_scheduler.shutdown();
    }

    fn enqueue(queue: &Arc<Mutex<QueueMap>>, target: QueueTarget, uris: &[&str]) {
        let mut guard = match queue.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = guard.entry(target).or_default();
        for uri in uris {
            entry.insert((*uri).to_string());
        }
    }
}

// --- Deserialization models for Spotify JSON responses ---

#[derive(Deserialize)]
struct SpotifyCurrentlyPlaying {
    context: Option<SpotifyContext>,
    item: Option<SpotifyTrackItem>,
}

#[derive(Deserialize)]
struct SpotifyContext {
    uri: String,
}

#[derive(Deserialize)]
struct SpotifyTrackItem {
    uri: String,
    #[serde(default)]
    is_local: bool,
}

#[derive(Deserialize)]
struct SpotifyPaginatedPlaylists {
    items: Vec<SpotifyPlaylistItem>,
    next: Option<String>,
}

#[derive(Deserialize)]
struct SpotifyPlaylistItem {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct SpotifyPlaylistDetails {
    snapshot_id: String,
    tracks: SpotifyPlaylistTracksPage,
}

#[derive(Deserialize)]
struct SpotifyPlaylistTracksPage {
    items: Vec<SpotifyPlaylistTrackItem>,
    next: Option<String>,
}

#[derive(Deserialize)]
struct SpotifyPlaylistTrackItem {
    #[serde(default)]
    is_local: bool,
    track: Option<SpotifyPlaylistTrack>,
}

#[derive(Deserialize)]
struct SpotifyPlaylistTrack {
    uri: Option<String>,
}

impl SpotifyApiClient for UreqSpotifyApiClient {
    fn get_currently_playing(&self) -> AppResult<Option<CurrentlyPlayingResponse>> {
        info!("Spotify API: get currently playing");
        let token = self.token()?;
        let url = self.url(SpotifyApiAction::CurrentlyPlaying)?;

        let mut response = self
            .playlist_scheduler
            .run("get currently playing", ScheduleMode::FailFast, || {
                ureq::get(&url)
                    .header("Authorization", &format!("Bearer {token}"))
                    .call()
            })
            .map_err(|e| {
                error!(error = %e, "Failed to get currently playing");
                e
            })?;

        if response.status().as_u16() == 204 {
            debug!("Spotify API: currently playing returned 204");
            return Ok(None);
        }

        let body: SpotifyCurrentlyPlaying = response.body_mut().read_json().map_err(|e| {
            error!(error = %e, "Failed to parse currently playing response");
            anyhow::anyhow!("Failed to parse currently playing response: {e}")
        })?;

        Ok(body.item.map(|item| CurrentlyPlayingResponse {
            context_uri: body.context.map(|c| c.uri),
            track_uri: item.uri,
            is_local: item.is_local,
        }))
    }

    fn get_my_playlists(&self) -> AppResult<Vec<PlaylistSummary>> {
        info!("Spotify API: get my playlists");
        let token = self.token()?;
        let base_url = self.url(SpotifyApiAction::MyPlaylists)?;
        let mut all = Vec::new();
        let mut offset = 0u32;

        loop {
            let url = format!("{base_url}?limit=50&offset={offset}");
            let page: SpotifyPaginatedPlaylists = self
                .schedule("get my playlists", || {
                    ureq::get(&url)
                        .header("Authorization", &format!("Bearer {token}"))
                        .call()
                })
                .map_err(|e| {
                    error!(error = %e, "Failed to get playlists");
                    e
                })?;

            all.extend(page.items.into_iter().map(|p| PlaylistSummary {
                id: p.id,
                name: p.name,
            }));

            if page.next.is_none() {
                break;
            }
            offset += 50;
        }

        Ok(all)
    }

    fn add_to_library(&self, uris: &[&str]) -> AppResult<()> {
        info!(count = uris.len(), "Queue add to liked songs");
        Self::enqueue(&self.add_queue, QueueTarget::Liked, uris);
        Ok(())
    }

    fn remove_from_library(&self, uris: &[&str]) -> AppResult<()> {
        info!(count = uris.len(), "Queue remove from liked songs");
        Self::enqueue(&self.remove_queue, QueueTarget::Liked, uris);
        Ok(())
    }

    fn add_to_playlist(&self, playlist_id: &str, uris: &[&str]) -> AppResult<()> {
        info!(playlist_id, count = uris.len(), "Queue add to playlist");
        Self::enqueue(
            &self.add_queue,
            QueueTarget::Playlist(playlist_id.to_string()),
            uris,
        );
        Ok(())
    }

    fn remove_from_playlist(&self, playlist_id: &str, uris: &[&str]) -> AppResult<()> {
        info!(
            playlist_id,
            count = uris.len(),
            "Queue remove from playlist"
        );
        Self::enqueue(
            &self.remove_queue,
            QueueTarget::Playlist(playlist_id.to_string()),
            uris,
        );
        Ok(())
    }

    fn remove_local_from_playlist(
        &self,
        playlist_id: &str,
        local_track_uri: &str,
    ) -> AppResult<()> {
        info!(
            playlist_id,
            local_track_uri, "Remove local track from playlist"
        );
        let token = self.token()?;
        let base_url = self
            .url(SpotifyApiAction::Playlist)?
            .replace("{id}", playlist_id);
        let mut snapshot_id = None;
        let mut position = None;
        let mut offset = 0usize;

        while position.is_none() {
            let url = format!(
                "{base_url}?fields=snapshot_id,tracks.items(is_local,track(uri)),tracks.next&limit=100&offset={offset}"
            );
            let page: SpotifyPlaylistDetails =
                self.schedule_playlist_wait("get playlist items for local removal", || {
                    ureq::get(&url)
                        .header("Authorization", &format!("Bearer {token}"))
                        .call()
                })?;

            if snapshot_id.is_none() {
                snapshot_id = Some(page.snapshot_id);
            }

            if let Some(index) = page.tracks.items.iter().position(|item| {
                item.is_local
                    && item.track.as_ref().and_then(|track| track.uri.as_deref())
                        == Some(local_track_uri)
            }) {
                position = Some(offset + index);
                break;
            }

            if page.tracks.next.is_none() {
                break;
            }

            offset += page.tracks.items.len();
        }

        let position = position.ok_or_else(|| {
            anyhow::anyhow!(
                "Local track '{local_track_uri}' was not found in playlist '{playlist_id}'"
            )
        })?;
        let snapshot_id = snapshot_id.ok_or_else(|| {
            anyhow::anyhow!("Playlist '{playlist_id}' snapshot id is unavailable")
        })?;
        let url = self
            .url(SpotifyApiAction::PlaylistItems)?
            .replace("{id}", playlist_id);
        let payload = json!({
            "items": [{ "uri": local_track_uri, "positions": [position] }],
            "snapshot_id": snapshot_id,
        });

        self.schedule_playlist_wait_empty("remove local track from playlist", || {
            ureq::delete(&url)
                .force_send_body()
                .header("Authorization", &format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .send_json(payload.clone())
        })?;

        Ok(())
    }

    fn skip_to_next(&self) -> AppResult<()> {
        info!("Spotify API: skip to next");
        let token = self.token()?;
        let url = self.url(SpotifyApiAction::NextTrack)?;

        self.schedule_playback("skip to next track", || {
            ureq::post(&url)
                .header("Authorization", &format!("Bearer {token}"))
                .header("Content-Length", "0")
                .send_empty()
        })
        .map_err(|e| {
            error!(error = %e, "Failed to skip to next track");
            e
        })?;

        Ok(())
    }
}
