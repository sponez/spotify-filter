#[derive(Debug, thiserror::Error)]
#[error("Invalid Spotify URI: '{0}'")]
pub struct SpotifyUriParseError(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpotifyUriType {
    Track,
    Playlist,
    User,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpotifyUserSubpath {
    Collection,
    Other,
}

pub struct SpotifyUri {
    pub uri_type: SpotifyUriType,
    pub id: String,
    pub user_subpath: Option<SpotifyUserSubpath>,
}
