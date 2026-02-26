use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use domain::{
    errors::errors::AppResult,
    ports::ports_out::{
        client::spotify_api::{
            CurrentlyPlayingResponse, PlaylistSummary, SpotifyApiClient,
        },
        notification::ErrorNotification,
        repository::token::TokenCache,
    },
};
use indexmap::IndexSet;
use serde::Deserialize;
use tracing::{debug, error, info};

use crate::adapters_out::client::spotify::{
    action::SpotifyApiAction,
    request_scheduler::{RequestScheduler, ScheduleMode},
};

const CRON_INTERVAL: Duration = Duration::from_secs(3600);
const PHASE_GAP: Duration = Duration::from_secs(35);
const PLAYLIST_API_INTERVAL: Duration = Duration::from_secs(35);
const PLAYBACK_API_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum QueueTarget {
    Liked,
    Playlist(String),
}

type QueueMap = HashMap<QueueTarget, IndexSet<String>>;

pub struct UreqSpotifyApiClient {
    base_url: String,
    paths: HashMap<SpotifyApiAction, String>,
    token_cache: Arc<dyn TokenCache>,
    notifier: Arc<dyn ErrorNotification>,
    playlist_scheduler: Arc<RequestScheduler>,
    playback_scheduler: Arc<RequestScheduler>,
    add_queue: Arc<Mutex<QueueMap>>,
    remove_queue: Arc<Mutex<QueueMap>>,
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

        let client = Self {
            base_url,
            paths,
            token_cache,
            notifier,
            playlist_scheduler: Arc::clone(&playlist_scheduler),
            playback_scheduler: Arc::clone(&playback_scheduler),
            add_queue: Arc::clone(&add_queue),
            remove_queue: Arc::clone(&remove_queue),
        };

        Self::start_cron(
            client.base_url.clone(),
            client.paths.clone(),
            Arc::clone(&client.token_cache),
            Arc::clone(&client.notifier),
            playlist_scheduler,
            add_queue,
            remove_queue,
        );

        client
    }

    fn token(&self) -> AppResult<String> {
        Ok(self.token_cache
            .load()
            .ok_or_else(|| anyhow::anyhow!("No access token available"))?)
    }

    fn url(&self, action: SpotifyApiAction) -> AppResult<String> {
        let path = self.paths.get(&action)
            .ok_or_else(|| anyhow::anyhow!("No path configured for action '{action:?}'"))?;
        Ok(format!("{}{}", self.base_url, path))
    }

    fn schedule<T, F>(&self, op_name: &str, op: F) -> AppResult<T>
    where
        F: FnMut() -> Result<T, ureq::Error>,
    {
        self.playlist_scheduler.run(op_name, ScheduleMode::FailFast, op)
    }

    fn schedule_playback<T, F>(&self, op_name: &str, op: F) -> AppResult<T>
    where
        F: FnMut() -> Result<T, ureq::Error>,
    {
        self.playback_scheduler.run(op_name, ScheduleMode::FailFast, op)
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

    fn drain_queue(queue: &Arc<Mutex<QueueMap>>) -> QueueMap {
        let mut guard = match queue.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        std::mem::take(&mut *guard)
    }

    fn merge_back(queue: &Arc<Mutex<QueueMap>>, failed: QueueMap) {
        if failed.is_empty() {
            return;
        }
        let mut guard = match queue.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        for (target, uris) in failed {
            let entry = guard.entry(target).or_default();
            for uri in uris {
                entry.insert(uri);
            }
        }
    }

    fn path(paths: &HashMap<SpotifyApiAction, String>, action: SpotifyApiAction) -> AppResult<String> {
        paths.get(&action)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No path configured for action '{action:?}'").into())
    }

    fn token_from_cache(token_cache: &Arc<dyn TokenCache>) -> AppResult<String> {
        token_cache
            .load()
            .ok_or_else(|| anyhow::anyhow!("No access token available").into())
    }

    fn run_add_batch(
        base_url: &str,
        paths: &HashMap<SpotifyApiAction, String>,
        token_cache: &Arc<dyn TokenCache>,
        scheduler: &Arc<RequestScheduler>,
        target: &QueueTarget,
        uris: &IndexSet<String>,
    ) -> AppResult<()> {
        if uris.is_empty() {
            return Ok(());
        }
        let token = Self::token_from_cache(token_cache)?;
        let ordered_uris: Vec<String> = uris.iter().cloned().collect();
        match target {
            QueueTarget::Liked => {
                let url = format!(
                    "{}{}?uris={}",
                    base_url,
                    Self::path(paths, SpotifyApiAction::Library)?,
                    ordered_uris.join(",")
                );
                scheduler.run("cron add to liked", ScheduleMode::Wait, || {
                    ureq::put(&url)
                        .set("Authorization", &format!("Bearer {token}"))
                        .call()
                })?;
            }
            QueueTarget::Playlist(playlist_id) => {
                let path = Self::path(paths, SpotifyApiAction::PlaylistItems)?.replace("{id}", playlist_id);
                let url = format!("{base_url}{path}");
                let payload_uris = ordered_uris.clone();
                scheduler.run("cron add to playlist", ScheduleMode::Wait, || {
                    ureq::post(&url)
                        .set("Authorization", &format!("Bearer {token}"))
                        .set("Content-Type", "application/json")
                        .send_json(ureq::json!({ "uris": payload_uris, "position": 0 }))
                })?;
            }
        }
        Ok(())
    }

    fn run_remove_batch(
        base_url: &str,
        paths: &HashMap<SpotifyApiAction, String>,
        token_cache: &Arc<dyn TokenCache>,
        scheduler: &Arc<RequestScheduler>,
        target: &QueueTarget,
        uris: &IndexSet<String>,
    ) -> AppResult<()> {
        if uris.is_empty() {
            return Ok(());
        }
        let token = Self::token_from_cache(token_cache)?;
        let ordered_uris: Vec<String> = uris.iter().cloned().collect();
        match target {
            QueueTarget::Liked => {
                let url = format!(
                    "{}{}?uris={}",
                    base_url,
                    Self::path(paths, SpotifyApiAction::Library)?,
                    ordered_uris.join(",")
                );
                scheduler.run("cron remove from liked", ScheduleMode::Wait, || {
                    ureq::request("DELETE", &url)
                        .set("Authorization", &format!("Bearer {token}"))
                        .call()
                })?;
            }
            QueueTarget::Playlist(playlist_id) => {
                let path = Self::path(paths, SpotifyApiAction::PlaylistItems)?.replace("{id}", playlist_id);
                let url = format!("{base_url}{path}");
                let tracks: Vec<_> = ordered_uris.iter().map(|u| ureq::json!({ "uri": u })).collect();
                scheduler.run("cron remove from playlist", ScheduleMode::Wait, || {
                    ureq::request("DELETE", &url)
                        .set("Authorization", &format!("Bearer {token}"))
                        .set("Content-Type", "application/json")
                        .send_json(ureq::json!({ "items": tracks }))
                })?;
            }
        }
        Ok(())
    }

    fn process_queue(
        base_url: &str,
        paths: &HashMap<SpotifyApiAction, String>,
        token_cache: &Arc<dyn TokenCache>,
        notifier: &Arc<dyn ErrorNotification>,
        scheduler: &Arc<RequestScheduler>,
        queue: &Arc<Mutex<QueueMap>>,
        is_add: bool,
    ) {
        let drained = Self::drain_queue(queue);
        if drained.is_empty() {
            return;
        }
        let mut failed = HashMap::new();
        for (target, uris) in drained {
            let result = if is_add {
                Self::run_add_batch(base_url, paths, token_cache, scheduler, &target, &uris)
            } else {
                Self::run_remove_batch(base_url, paths, token_cache, scheduler, &target, &uris)
            };
            if let Err(e) = result {
                error!(error = %e, ?target, "Failed to process playlist queue batch");
                notifier.notify(&e.to_string());
                failed.insert(target, uris);
            }
        }
        Self::merge_back(queue, failed);
    }

    fn start_cron(
        base_url: String,
        paths: HashMap<SpotifyApiAction, String>,
        token_cache: Arc<dyn TokenCache>,
        notifier: Arc<dyn ErrorNotification>,
        scheduler: Arc<RequestScheduler>,
        add_queue: Arc<Mutex<QueueMap>>,
        remove_queue: Arc<Mutex<QueueMap>>,
    ) {
        std::thread::spawn(move || loop {
            std::thread::sleep(CRON_INTERVAL);
            notifier.notify("Playlist sync started");

            Self::process_queue(
                &base_url,
                &paths,
                &token_cache,
                &notifier,
                &scheduler,
                &add_queue,
                true,
            );

            std::thread::sleep(PHASE_GAP);

            Self::process_queue(
                &base_url,
                &paths,
                &token_cache,
                &notifier,
                &scheduler,
                &remove_queue,
                false,
            );
        });
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

impl SpotifyApiClient for UreqSpotifyApiClient {
    fn get_currently_playing(&self) -> AppResult<Option<CurrentlyPlayingResponse>> {
        info!("Spotify API: get currently playing");
        let token = self.token()?;
        let url = self.url(SpotifyApiAction::CurrentlyPlaying)?;

        let response = self.schedule("get currently playing", || {
            ureq::get(&url)
                .set("Authorization", &format!("Bearer {token}"))
                .call()
        }).map_err(|e| {
            error!(error = %e, "Failed to get currently playing");
            e
        })?;

        if response.status() == 204 {
            debug!("Spotify API: currently playing returned 204");
            return Ok(None);
        }

        let body: SpotifyCurrentlyPlaying = response
            .into_json()
            .map_err(|e| {
                error!(error = %e, "Failed to parse currently playing response");
                anyhow::anyhow!("Failed to parse currently playing response: {e}")
            })?;

        Ok(body.item.map(|item| CurrentlyPlayingResponse {
            context_uri: body.context.map(|c| c.uri),
            track_uri: item.uri,
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
            let page: SpotifyPaginatedPlaylists = self.schedule("get my playlists", || {
                ureq::get(&url)
                    .set("Authorization", &format!("Bearer {token}"))
                    .call()
            }).map_err(|e| {
                error!(error = %e, "Failed to get playlists");
                e
            })?
                .into_json()
                .map_err(|e| {
                    error!(error = %e, "Failed to parse playlists response");
                    anyhow::anyhow!("Failed to parse playlists response: {e}")
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
        Self::enqueue(&self.add_queue, QueueTarget::Playlist(playlist_id.to_string()), uris);
        Ok(())
    }

    fn remove_from_playlist(
        &self,
        playlist_id: &str,
        uris: &[&str],
    ) -> AppResult<()> {
        info!(playlist_id, count = uris.len(), "Queue remove from playlist");
        Self::enqueue(&self.remove_queue, QueueTarget::Playlist(playlist_id.to_string()), uris);
        Ok(())
    }

    fn skip_to_next(&self) -> AppResult<()> {
        info!("Spotify API: skip to next");
        let token = self.token()?;
        let url = self.url(SpotifyApiAction::NextTrack)?;

        self.schedule_playback("skip to next track", || {
            ureq::post(&url)
                .set("Authorization", &format!("Bearer {token}"))
                .set("Content-Length", "0")
                .call()
        }).map_err(|e| {
            error!(error = %e, "Failed to skip to next track");
            e
        })?;

        Ok(())
    }
}
