use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use domain::{
    errors::errors::AppResult,
    ports::ports_out::{
        client::spotify_api::{
            CurrentlyPlayingResponse, PlaylistSnapshotResponse, PlaylistSummary, SpotifyApiClient,
        },
        repository::token::TokenCache,
    },
};
use serde::Deserialize;
use tracing::{debug, error, info, warn};

pub struct UreqSpotifyApiClient {
    base_url: String,
    paths: HashMap<String, String>,
    token_cache: Arc<dyn TokenCache>,
    rate_state: Mutex<RateState>,
    snapshot_cache: Mutex<HashMap<String, CachedSnapshot>>,
}

struct RateState {
    next_allowed_at: Instant,
}

struct CachedSnapshot {
    snapshot_id: String,
    cached_at: Instant,
}

const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(350);
const DEFAULT_RETRY_AFTER: Duration = Duration::from_secs(3);
const SNAPSHOT_CACHE_TTL: Duration = Duration::from_secs(300);

impl UreqSpotifyApiClient {
    pub fn new(
        base_url: String,
        paths: HashMap<String, String>,
        token_cache: Arc<dyn TokenCache>,
    ) -> Self {
        Self {
            base_url,
            paths,
            token_cache,
            rate_state: Mutex::new(RateState {
                next_allowed_at: Instant::now(),
            }),
            snapshot_cache: Mutex::new(HashMap::new()),
        }
    }

    fn token(&self) -> AppResult<String> {
        Ok(self.token_cache
            .load()
            .ok_or_else(|| anyhow::anyhow!("No access token available"))?)
    }

    fn url(&self, action: &str) -> AppResult<String> {
        let path = self.paths.get(action)
            .ok_or_else(|| anyhow::anyhow!("No path configured for action '{action}'"))?;
        Ok(format!("{}{}", self.base_url, path))
    }

    fn url_with_id(&self, action: &str, id: &str) -> AppResult<String> {
        let path = self.paths.get(action)
            .ok_or_else(|| anyhow::anyhow!("No path configured for action '{action}'"))?;
        Ok(format!("{}{}", self.base_url, path.replace("{id}", id)))
    }

    fn try_acquire_rate_slot(&self, op_name: &str) -> AppResult<()> {
        let mut state = match self.rate_state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let now = Instant::now();
        if state.next_allowed_at > now {
            let wait = state.next_allowed_at.saturating_duration_since(now);
            warn!(
                operation = op_name,
                retry_after_secs = wait.as_secs(),
                "Request blocked by local cooldown"
            );
            return Err(anyhow::anyhow!(
                "Spotify cooldown active for operation '{op_name}', retry in {}s",
                wait.as_secs()
            ).into());
        }
        state.next_allowed_at = Instant::now() + MIN_REQUEST_INTERVAL;
        debug!(operation = op_name, "Rate limiter slot acquired");
        Ok(())
    }

    fn defer_requests(&self, wait: Duration) {
        let mut state = match self.rate_state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        state.next_allowed_at = std::cmp::max(state.next_allowed_at, Instant::now() + wait);
    }

    fn parse_retry_after(resp: &ureq::Response) -> Duration {
        let secs = resp
            .header("Retry-After")
            .and_then(|h| h.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_RETRY_AFTER.as_secs());
        Duration::from_secs(secs.max(1))
    }

    fn load_snapshot_from_cache(&self, playlist_id: &str) -> Option<String> {
        let mut cache = match self.snapshot_cache.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let now = Instant::now();
        if let Some(entry) = cache.get(playlist_id) {
            if now.saturating_duration_since(entry.cached_at) < SNAPSHOT_CACHE_TTL {
                return Some(entry.snapshot_id.clone());
            }
        }
        cache.remove(playlist_id);
        None
    }

    fn store_snapshot_in_cache(&self, playlist_id: &str, snapshot_id: &str) {
        let mut cache = match self.snapshot_cache.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        cache.insert(
            playlist_id.to_string(),
            CachedSnapshot {
                snapshot_id: snapshot_id.to_string(),
                cached_at: Instant::now(),
            },
        );
    }

    fn remove_snapshot_from_cache(&self, playlist_id: &str) {
        let mut cache = match self.snapshot_cache.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        cache.remove(playlist_id);
    }

    fn request_with_retry<T, F>(&self, op_name: &str, mut op: F) -> AppResult<T>
    where
        F: FnMut() -> Result<T, ureq::Error>,
    {
        self.try_acquire_rate_slot(op_name)?;
        match op() {
            Ok(value) => Ok(value),
            Err(ureq::Error::Status(429, response)) => {
                let wait = Self::parse_retry_after(&response);
                self.defer_requests(wait);
                warn!(
                    operation = op_name,
                    retry_after_secs = wait.as_secs(),
                    "Spotify returned 429 Too Many Requests"
                );
                Err(anyhow::anyhow!(
                    "{op_name} rate-limited by Spotify; retry in {}s",
                    wait.as_secs()
                ).into())
            }
            Err(ureq::Error::Status(status, response)) => {
                let status_text = response.status_text().to_string();
                Err(anyhow::anyhow!(
                    "{op_name} failed with status {status} {status_text}"
                ).into())
            }
            Err(ureq::Error::Transport(e)) => {
                Err(anyhow::anyhow!("{op_name} transport error: {e}").into())
            }
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
}

#[derive(Deserialize)]
struct SpotifyPlaylistSnapshot {
    snapshot_id: String,
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
        let url = self.url("currently-playing")?;

        let response = self.request_with_retry("get currently playing", || {
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

    fn get_playlist_snapshot(&self, playlist_id: &str) -> AppResult<PlaylistSnapshotResponse> {
        if let Some(snapshot_id) = self.load_snapshot_from_cache(playlist_id) {
            debug!(playlist_id, "Playlist snapshot cache hit");
            return Ok(PlaylistSnapshotResponse { snapshot_id });
        }
        info!(playlist_id, "Spotify API: get playlist snapshot");
        let token = self.token()?;
        let url = format!("{}?fields=snapshot_id", self.url_with_id("playlist", playlist_id)?);

        let body: SpotifyPlaylistSnapshot = self.request_with_retry("get playlist snapshot", || {
            ureq::get(&url)
                .set("Authorization", &format!("Bearer {token}"))
                .call()
        }).map_err(|e| {
            error!(error = %e, "Failed to get playlist snapshot");
            e
        })?
            .into_json()
            .map_err(|e| {
                error!(error = %e, "Failed to parse playlist snapshot response");
                anyhow::anyhow!("Failed to parse playlist snapshot response: {e}")
            })?;
        self.store_snapshot_in_cache(playlist_id, &body.snapshot_id);

        Ok(PlaylistSnapshotResponse {
            snapshot_id: body.snapshot_id,
        })
    }

    fn get_my_playlists(&self) -> AppResult<Vec<PlaylistSummary>> {
        info!("Spotify API: get my playlists");
        let token = self.token()?;
        let base_url = self.url("my-playlists")?;
        let mut all = Vec::new();
        let mut offset = 0u32;

        loop {
            let url = format!("{base_url}?limit=50&offset={offset}");
            let page: SpotifyPaginatedPlaylists = self.request_with_retry("get my playlists", || {
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
        info!(count = uris.len(), "Spotify API: add to library");
        let token = self.token()?;
        let ids = uris.join(",");
        let url = format!("{}?uris={ids}", self.url("library")?);

        self.request_with_retry("add to library", || {
            ureq::put(&url)
                .set("Authorization", &format!("Bearer {token}"))
                .call()
        }).map_err(|e| {
            error!(error = %e, "Failed to add to library");
            e
        })?;

        Ok(())
    }

    fn remove_from_library(&self, uris: &[&str]) -> AppResult<()> {
        info!(count = uris.len(), "Spotify API: remove from library");
        let token = self.token()?;
        let ids = uris.join(",");
        let url = format!("{}?uris={ids}", self.url("library")?);

        self.request_with_retry("remove from library", || {
            ureq::request("DELETE", &url)
                .set("Authorization", &format!("Bearer {token}"))
                .call()
        }).map_err(|e| {
            error!(error = %e, "Failed to remove from library");
            e
        })?;

        Ok(())
    }

    fn add_to_playlist(&self, playlist_id: &str, uris: &[&str]) -> AppResult<()> {
        info!(playlist_id, count = uris.len(), "Spotify API: add to playlist");
        let token = self.token()?;
        let url = self.url_with_id("playlist-items", playlist_id)?;

        self.request_with_retry("add to playlist", || {
            ureq::post(&url)
                .set("Authorization", &format!("Bearer {token}"))
                .set("Content-Type", "application/json")
                .send_json(ureq::json!({ "uris": uris, "position": 0 }))
        }).map_err(|e| {
            error!(error = %e, "Failed to add to playlist");
            e
        })?;

        Ok(())
    }

    fn remove_from_playlist(
        &self,
        playlist_id: &str,
        uris: &[&str],
        snapshot_id: &str,
    ) -> AppResult<()> {
        info!(playlist_id, count = uris.len(), "Spotify API: remove from playlist");
        let token = self.token()?;
        let url = self.url_with_id("playlist-items", playlist_id)?;

        let tracks: Vec<_> = uris.iter().map(|u| ureq::json!({ "uri": u })).collect();

        let response = self.request_with_retry("remove from playlist", || {
            ureq::request("DELETE", &url)
                .set("Authorization", &format!("Bearer {token}"))
                .set("Content-Type", "application/json")
                .send_json(ureq::json!({
                    "items": tracks,
                    "snapshot_id": snapshot_id,
                }))
        }).map_err(|e| {
            error!(error = %e, "Failed to remove from playlist");
            e
        })?;

        match response.into_json::<SpotifyPlaylistSnapshot>() {
            Ok(body) => {
                self.store_snapshot_in_cache(playlist_id, &body.snapshot_id);
                debug!(playlist_id, "Updated playlist snapshot cache from delete response");
            }
            Err(e) => {
                warn!(
                    playlist_id,
                    error = %e,
                    "Failed to parse snapshot from delete response, invalidating snapshot cache"
                );
                self.remove_snapshot_from_cache(playlist_id);
            }
        }

        Ok(())
    }

    fn skip_to_next(&self) -> AppResult<()> {
        info!("Spotify API: skip to next");
        let token = self.token()?;
        let url = self.url("next-track")?;

        self.request_with_retry("skip to next track", || {
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
