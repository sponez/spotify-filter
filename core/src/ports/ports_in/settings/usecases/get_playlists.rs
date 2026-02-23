use crate::ports::ports_in::settings::models::PlaylistItemView;

pub trait GetPlaylistsUseCase: Send + Sync {
    fn get_playlists(&self) -> Vec<PlaylistItemView>;
}
