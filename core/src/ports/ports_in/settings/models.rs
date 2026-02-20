/// Read model returned by GetSettingsUseCase.
#[derive(Clone)]
pub struct SettingsView {
    pub filter_action: FilterActionView,
    pub filter_target: FilterTargetView,
}

#[derive(Clone, PartialEq)]
pub enum FilterActionView {
    None,
    AddToPlaylist,
    MoveToPlaylist,
}

#[derive(Clone, PartialEq)]
pub enum FilterTargetView {
    LikedSongs,
    Playlist(String),
}

/// Command carried by SaveSettingsUseCase.
pub struct SaveSettingsCommand {
    pub filter_action: FilterActionView,
    pub filter_target: FilterTargetView,
}
