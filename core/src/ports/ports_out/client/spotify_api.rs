use crate::errors::errors::AppResult;

pub struct CurrentlyPlayingResponse {
    pub track_name: String,
    pub artist_names: Vec<String>,
    pub track_uri: String,
}

pub trait SpotifyApiClient: Send + Sync {
    fn get_currently_playing(&self) -> AppResult<Option<CurrentlyPlayingResponse>>;
}
