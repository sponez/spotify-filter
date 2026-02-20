use global_hotkey::hotkey::{Code, HotKey, Modifiers};

/// Application configuration.
/// Hardcoded for now — can be made user-configurable later.
pub struct AppConfig {
    /// Hotkey: remove current track from playlist and skip.
    pub hotkey_discard: HotKey,
    /// Hotkey: like current track, remove from playlist, and skip.
    pub hotkey_like: HotKey,
    /// Spotify OAuth2 client ID.
    pub spotify_client_id: String,
    /// OAuth2 redirect URI (must match the Spotify app settings).
    pub spotify_redirect_uri: String,
}

impl AppConfig {
    pub fn default_config() -> Self {
        Self {
            hotkey_discard: HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyD),
            hotkey_like: HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyL),
            spotify_client_id: String::new(),     // TODO: fill in from env or secure store
            spotify_redirect_uri: "http://localhost:8888/callback".to_string(),
        }
    }
}
