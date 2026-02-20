/// Stub Spotify API client.
/// Implements [`SpotifyGateway`] — real HTTP calls will be added here later.
pub struct SpotifyClient {}

impl SpotifyGateway for SpotifyClient {
    fn remove_from_playlist(&self, track_uri: &str, playlist_id: &str) {
        println!("[spotify] remove_from_playlist: {track_uri} from {playlist_id}");
        // TODO: DELETE https://api.spotify.com/v1/playlists/{id}/tracks
    }

    fn like_track(&self, track_uri: &str) {
        println!("[spotify] like_track: {track_uri}");
        // TODO: PUT https://api.spotify.com/v1/me/tracks
    }

    fn skip_next(&self) {
        println!("[spotify] skip_next");
        // TODO: POST https://api.spotify.com/v1/me/player/next
    }

    fn get_current_track(&self) -> Option<Track> {
        println!("[spotify] get_current_track");
        // TODO: GET https://api.spotify.com/v1/me/player/currently-playing
        None
    }
}
