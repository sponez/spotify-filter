use std::collections::HashMap;
use std::sync::Arc;

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

use crate::adapters_out::client::spotify::{
    request_scheduler::RequestScheduler,
    snapshot_cache::SnapshotCache,
};

pub struct UreqSpotifyApiClient {
    base_url: String,
    paths: HashMap<String, String>,
    token_cache: Arc<dyn TokenCache>,
    scheduler: RequestScheduler,
    snapshots: SnapshotCache,
}

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
            scheduler: RequestScheduler::new(),
            snapshots: SnapshotCache::new(),
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

    fn schedule<T, F>(&self, op_name: &str, op: F) -> AppResult<T>
    where
        F: FnMut() -> Result<T, ureq::Error>,
    {
        self.scheduler.run(op_name, op)
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

    fn get_playlist_snapshot(&self, playlist_id: &str) -> AppResult<PlaylistSnapshotResponse> {
        if let Some(snapshot_id) = self.snapshots.get(playlist_id) {
            debug!(playlist_id, "Playlist snapshot cache hit");
            return Ok(PlaylistSnapshotResponse { snapshot_id });
        }
        info!(playlist_id, "Spotify API: get playlist snapshot");
        let token = self.token()?;
        let url = format!("{}?fields=snapshot_id", self.url_with_id("playlist", playlist_id)?);

        let body: SpotifyPlaylistSnapshot = self.schedule("get playlist snapshot", || {
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
        self.snapshots.put(playlist_id, &body.snapshot_id);

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
        info!(count = uris.len(), "Spotify API: add to library");
        let token = self.token()?;
        let ids = uris.join(",");
        let url = format!("{}?uris={ids}", self.url("library")?);

        self.schedule("add to library", || {
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

        self.schedule("remove from library", || {
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

        self.schedule("add to playlist", || {
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

        let response = self.schedule("remove from playlist", || {
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
                self.snapshots.put(playlist_id, &body.snapshot_id);
                debug!(playlist_id, "Updated playlist snapshot cache from delete response");
            }
            Err(e) => {
                warn!(
                    playlist_id,
                    error = %e,
                    "Failed to parse snapshot from delete response, invalidating snapshot cache"
                );
                self.snapshots.invalidate(playlist_id);
            }
        }

        Ok(())
    }

    fn skip_to_next(&self) -> AppResult<()> {
        info!("Spotify API: skip to next");
        let token = self.token()?;
        let url = self.url("next-track")?;

        self.schedule("skip to next track", || {
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
