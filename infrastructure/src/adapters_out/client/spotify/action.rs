#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SpotifyApiAction {
    CurrentlyPlaying,
    MyPlaylists,
    Library,
    Playlist,
    PlaylistItems,
    NextTrack,
}
