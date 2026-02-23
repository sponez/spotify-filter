use serde::Deserialize;
use tracing::{debug, info};

use crate::configuration::models::hotkeys::HotkeysConfig;
use crate::configuration::models::spotify::SpotifyConfig;

#[derive(Deserialize)]
pub struct AppConfig {
    pub spotify: SpotifyConfig,
}

#[derive(Deserialize)]
pub struct Configuration {
    pub app: AppConfig,
    pub hotkeys: HotkeysConfig,
}

impl Configuration {
    pub fn load() -> Self {
        info!("Loading configuration");
        let exe = std::env::current_exe().expect("cannot determine executable path");
        let dir = exe.parent().expect("executable has no parent directory");

        if dotenvy::from_path(dir.join(".env")).is_err() {
            dotenvy::dotenv().ok();
        }

        let contents = std::fs::read_to_string(dir.join("configuration.toml"))
            .or_else(|_| std::fs::read_to_string("configuration.toml"))
            .expect("cannot read configuration.toml");

        let mut config: Self = toml::from_str(&contents)
            .unwrap_or_else(|e| panic!("invalid configuration.toml: {e}"));

        config.app.spotify.auth.client_id = std::env::var("SPOTIFY_CLIENT_ID")
            .expect("SPOTIFY_CLIENT_ID must be set in .env");
        config.app.spotify.auth.client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
            .expect("SPOTIFY_CLIENT_SECRET must be set in .env");

        debug!(
            redirect_uri = %config.app.spotify.auth.redirect_uri,
            scopes = config.app.spotify.auth.scopes.len(),
            "Configuration loaded"
        );
        config
    }
}
