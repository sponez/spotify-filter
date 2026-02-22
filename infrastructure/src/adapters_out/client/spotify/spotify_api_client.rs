use std::sync::Arc;

use domain::{
    errors::errors::AppResult,
    ports::ports_out::{
        client::spotify_api::{CurrentlyPlayingResponse, SpotifyApiClient},
        repository::token::TokenCache,
    },
};
use serde::Deserialize;

pub struct UreqSpotifyApiClient {
    base_url: String,
    token_cache: Arc<dyn TokenCache>,
}

impl UreqSpotifyApiClient {
    pub fn new(base_url: String, token_cache: Arc<dyn TokenCache>) -> Self {
        Self { base_url, token_cache }
    }
}

#[derive(Deserialize)]
struct SpotifyCurrentlyPlaying {
    item: Option<SpotifyTrackItem>,
}

#[derive(Deserialize)]
struct SpotifyTrackItem {
    name: String,
    uri: String,
    artists: Vec<SpotifyArtist>,
}

#[derive(Deserialize)]
struct SpotifyArtist {
    name: String,
}

impl SpotifyApiClient for UreqSpotifyApiClient {
    fn get_currently_playing(&self) -> AppResult<Option<CurrentlyPlayingResponse>> {
        let token = self.token_cache.load()
            .ok_or_else(|| anyhow::anyhow!("No access token available"))?;

        let url = format!("{}v1/me/player/currently-playing", self.base_url);
        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .call()
            .map_err(|e| anyhow::anyhow!("Failed to get currently playing: {e}"))?;

        if response.status() == 204 {
            return Ok(None);
        }

        let body: SpotifyCurrentlyPlaying = response.into_json()
            .map_err(|e| anyhow::anyhow!("Failed to parse currently playing response: {e}"))?;

        Ok(body.item.map(|item| CurrentlyPlayingResponse {
            track_name: item.name,
            artist_names: item.artists.into_iter().map(|a| a.name).collect(),
            track_uri: item.uri,
        }))
    }
}
