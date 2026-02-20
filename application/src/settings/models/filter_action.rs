use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AdditionalFilterAction {
    #[default]
    None,
    AddToPlaylist,
    MoveToPlaylist,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlaylistTarget {
    Liked,
    Playlist(String),
}
