use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SpotifyAction {
    GetTrack,
}

#[derive(Deserialize)]
pub struct SpotifyConfig {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    pub url: String,
    pub redirect_uri: String,
    pub paths: HashMap<SpotifyAction, String>,
}
