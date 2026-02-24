use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SpotifyAction {
    CurrentlyPlaying,
    MyPlaylists,
    Library,
    PlaylistItems,
    NextTrack,
}

#[derive(Deserialize)]
pub struct SpotifyConfig {
    pub api: SpotifyApiConfig,
    pub auth: SpotifyAuthConfig,
}

#[derive(Deserialize)]
pub struct SpotifyApiConfig {
    pub url: String,
    pub paths: HashMap<SpotifyAction, String>,
}

#[derive(Deserialize)]
pub struct SpotifyAuthConfig {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}
