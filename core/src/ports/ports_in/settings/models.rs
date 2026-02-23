/// Read model returned by GetSettingsUseCase.
#[derive(Clone)]
pub struct SettingsView {
    pub pass_action: PassActionView,
    pub pass_target: PassTargetView,
}

#[derive(Clone)]
pub struct PlaylistItemView {
    pub id: String,
    pub name: String,
}

#[derive(Clone, PartialEq)]
pub enum PassActionView {
    None,
    AddToPlaylist,
    MoveToPlaylist,
}

#[derive(Clone, PartialEq)]
pub enum PassTargetView {
    LikedSongs,
    Playlist(String),
}

/// Command carried by SaveSettingsUseCase.
pub struct SaveSettingsCommand {
    pub filter_action: PassActionView,
    pub filter_target: PassTargetView,
}
