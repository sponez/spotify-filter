use crate::{errors::errors::AppResult, ports::ports_in::settings::models::PlaylistItemView};

pub trait GetPlaylistsUseCase: Send + Sync {
    fn get_playlists(&self) -> AppResult<Vec<PlaylistItemView>>;
}
