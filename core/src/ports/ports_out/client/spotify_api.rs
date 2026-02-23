use crate::errors::errors::AppResult;

pub struct CurrentlyPlayingResponse {
    pub context_uri: Option<String>,
    pub track_uri: String,
}

pub struct PlaylistSnapshotResponse {
    pub snapshot_id: String,
}

pub struct PlaylistSummary {
    pub id: String,
    pub name: String,
}

pub trait SpotifyApiClient: Send + Sync {
    fn get_currently_playing(&self) -> AppResult<Option<CurrentlyPlayingResponse>>;
    fn get_playlist_snapshot(&self, playlist_id: &str) -> AppResult<PlaylistSnapshotResponse>;
    fn get_my_playlists(&self) -> AppResult<Vec<PlaylistSummary>>;
    fn add_to_library(&self, uris: &[&str]) -> AppResult<()>;
    fn remove_from_library(&self, uris: &[&str]) -> AppResult<()>;
    fn add_to_playlist(&self, playlist_id: &str, uris: &[&str]) -> AppResult<()>;
    fn remove_from_playlist(&self, playlist_id: &str, uris: &[&str], snapshot_id: &str) -> AppResult<()>;
    fn skip_to_next(&self) -> AppResult<()>;
}
