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

pub struct UreqSpotifyApiClient {
    base_url: String,
    paths: HashMap<String, String>,
    token_cache: Arc<dyn TokenCache>,
}

impl UreqSpotifyApiClient {
    pub fn new(
        base_url: String,
        paths: HashMap<String, String>,
        token_cache: Arc<dyn TokenCache>,
    ) -> Self {
        Self { base_url, paths, token_cache }
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
        let token = self.token()?;
        let url = self.url("currently-playing")?;

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .call()
            .map_err(|e| anyhow::anyhow!("Failed to get currently playing: {e}"))?;

        if response.status() == 204 {
            return Ok(None);
        }

        let body: SpotifyCurrentlyPlaying = response
            .into_json()
            .map_err(|e| anyhow::anyhow!("Failed to parse currently playing response: {e}"))?;

        Ok(body.item.map(|item| CurrentlyPlayingResponse {
            context_uri: body.context.map(|c| c.uri),
            track_uri: item.uri,
        }))
    }

    fn get_playlist_snapshot(&self, playlist_id: &str) -> AppResult<PlaylistSnapshotResponse> {
        let token = self.token()?;
        let url = format!("{}?fields=snapshot_id", self.url_with_id("playlist", playlist_id)?);

        let body: SpotifyPlaylistSnapshot = ureq::get(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .call()
            .map_err(|e| anyhow::anyhow!("Failed to get playlist snapshot: {e}"))?
            .into_json()
            .map_err(|e| anyhow::anyhow!("Failed to parse playlist snapshot response: {e}"))?;

        Ok(PlaylistSnapshotResponse {
            snapshot_id: body.snapshot_id,
        })
    }

    fn get_my_playlists(&self) -> AppResult<Vec<PlaylistSummary>> {
        let token = self.token()?;
        let base_url = self.url("my-playlists")?;
        let mut all = Vec::new();
        let mut offset = 0u32;

        loop {
            let url = format!("{base_url}?limit=50&offset={offset}");
            let page: SpotifyPaginatedPlaylists = ureq::get(&url)
                .set("Authorization", &format!("Bearer {token}"))
                .call()
                .map_err(|e| anyhow::anyhow!("Failed to get playlists: {e}"))?
                .into_json()
                .map_err(|e| anyhow::anyhow!("Failed to parse playlists response: {e}"))?;

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
        let token = self.token()?;
        let ids = uris.join(",");
        let url = format!("{}?uris={ids}", self.url("library")?);

        ureq::put(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .call()
            .map_err(|e| anyhow::anyhow!("Failed to add to library: {e}"))?;

        Ok(())
    }

    fn remove_from_library(&self, uris: &[&str]) -> AppResult<()> {
        let token = self.token()?;
        let ids = uris.join(",");
        let url = format!("{}?uris={ids}", self.url("library")?);

        ureq::request("DELETE", &url)
            .set("Authorization", &format!("Bearer {token}"))
            .call()
            .map_err(|e| anyhow::anyhow!("Failed to remove from library: {e}"))?;

        Ok(())
    }

    fn add_to_playlist(&self, playlist_id: &str, uris: &[&str]) -> AppResult<()> {
        let token = self.token()?;
        let url = self.url_with_id("playlist-items", playlist_id)?;

        ureq::post(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .set("Content-Type", "application/json")
            .send_json(ureq::json!({ "uris": uris, "position": 0 }))
            .map_err(|e| anyhow::anyhow!("Failed to add to playlist: {e}"))?;

        Ok(())
    }

    fn remove_from_playlist(
        &self,
        playlist_id: &str,
        uris: &[&str],
        snapshot_id: &str,
    ) -> AppResult<()> {
        let token = self.token()?;
        let url = self.url_with_id("playlist-items", playlist_id)?;

        let tracks: Vec<_> = uris.iter().map(|u| ureq::json!({ "uri": u })).collect();

        ureq::request("DELETE", &url)
            .set("Authorization", &format!("Bearer {token}"))
            .set("Content-Type", "application/json")
            .send_json(ureq::json!({
                "items": tracks,
                "snapshot_id": snapshot_id,
            }))
            .map_err(|e| anyhow::anyhow!("Failed to remove from playlist: {e}"))?;

        Ok(())
    }

    fn skip_to_next(&self) -> AppResult<()> {
        let token = self.token()?;
        let url = self.url("next-track")?;

        ureq::post(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .set("Content-Length", "0")
            .call()
            .map_err(|e| anyhow::anyhow!("Failed to skip to next track: {e}"))?;

        Ok(())
    }
}
