use serde::{Deserialize, Serialize};

// ---- Cache DTO ----

#[derive(Clone, Default)]
pub struct SettingsCacheDto {
    pub filter_action: FilterActionDto,
    pub filter_target: Option<PlaylistTargetDto>,
}

// ---- File DTO ----

#[derive(Serialize, Deserialize, Default)]
pub struct SettingsFileDto {
    #[serde(default)]
    pub filter_action: FilterActionDto,
    pub filter_target: Option<PlaylistTargetDto>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FilterActionDto {
    #[default]
    None,
    AddToPlaylist,
    MoveToPlaylist,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlaylistTargetDto {
    Liked,
    Playlist(String),
}
