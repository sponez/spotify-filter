use std::sync::Arc;
use tracing::{error, info};

use crate::ports::{
    ports_in::settings::{
        models::PlaylistItemView,
        usecases::get_playlists::GetPlaylistsUseCase,
    },
    ports_out::client::spotify_api::SpotifyApiClient,
};

pub struct GetPlaylistsInteractor {
    api_client: Arc<dyn SpotifyApiClient>,
}

impl GetPlaylistsInteractor {
    pub fn new(api_client: Arc<dyn SpotifyApiClient>) -> Self {
        Self { api_client }
    }
}

impl GetPlaylistsUseCase for GetPlaylistsInteractor {
    fn get_playlists(&self) -> Vec<PlaylistItemView> {
        info!("Loading playlists from Spotify");
        self.api_client
            .get_my_playlists()
            .map_err(|e| {
                error!(error = %e, "Failed to load playlists");
                e
            })
            .unwrap_or_default()
            .into_iter()
            .map(|p| PlaylistItemView { id: p.id, name: p.name })
            .collect()
    }
}
