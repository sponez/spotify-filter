use crate::{
    domain::models::spotify_uri::{
        SpotifyUri, SpotifyUriParseError, SpotifyUriType, SpotifyUserSubpath,
    },
    errors::errors::AppResult,
};

pub fn parse_spotify_uri(uri: &str) -> AppResult<SpotifyUri> {
    let parts: Vec<&str> = uri.split(':').collect();

    if parts.len() < 3 || parts[0] != "spotify" {
        return Err(SpotifyUriParseError(uri.to_string()).into());
    }

    match parts[1] {
        "track" => Ok(SpotifyUri {
            uri_type: SpotifyUriType::Track,
            id: parts[2].to_string(),
            user_subpath: None,
        }),
        "playlist" => Ok(SpotifyUri {
            uri_type: SpotifyUriType::Playlist,
            id: parts[2].to_string(),
            user_subpath: None,
        }),
        "user" => {
            let subpath = if parts.len() >= 4 && parts[3] == "collection" {
                Some(SpotifyUserSubpath::Collection)
            } else if parts.len() >= 4 {
                Some(SpotifyUserSubpath::Other)
            } else {
                None
            };

            Ok(SpotifyUri {
                uri_type: SpotifyUriType::User,
                id: parts[2].to_string(),
                user_subpath: subpath,
            })
        }
        _ => Ok(SpotifyUri {
            uri_type: SpotifyUriType::Other,
            id: parts[2].to_string(),
            user_subpath: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_track_uri() {
        let uri = parse_spotify_uri("spotify:track:6rqhFgbbKwnb9MLmUQDhG6").unwrap();
        assert!(matches!(uri.uri_type, SpotifyUriType::Track));
        assert_eq!(uri.id, "6rqhFgbbKwnb9MLmUQDhG6");
        assert!(uri.user_subpath.is_none());
    }

    #[test]
    fn parses_playlist_uri() {
        let uri = parse_spotify_uri("spotify:playlist:37i9dQZF1DXcBWIGoYBM5M").unwrap();
        assert!(matches!(uri.uri_type, SpotifyUriType::Playlist));
        assert_eq!(uri.id, "37i9dQZF1DXcBWIGoYBM5M");
        assert!(uri.user_subpath.is_none());
    }

    #[test]
    fn parses_user_collection_uri() {
        let uri = parse_spotify_uri("spotify:user:12345:collection").unwrap();
        assert!(matches!(uri.uri_type, SpotifyUriType::User));
        assert_eq!(uri.id, "12345");
        assert!(matches!(
            uri.user_subpath,
            Some(SpotifyUserSubpath::Collection)
        ));
    }

    #[test]
    fn parses_user_uri_without_subpath() {
        let uri = parse_spotify_uri("spotify:user:12345").unwrap();
        assert!(matches!(uri.uri_type, SpotifyUriType::User));
        assert_eq!(uri.id, "12345");
        assert!(uri.user_subpath.is_none());
    }

    #[test]
    fn parses_other_uri_type() {
        let uri = parse_spotify_uri("spotify:album:4aawyAB9vmqN3uQ7FjRGTy").unwrap();
        assert!(matches!(uri.uri_type, SpotifyUriType::Other));
        assert_eq!(uri.id, "4aawyAB9vmqN3uQ7FjRGTy");
    }

    #[test]
    fn rejects_invalid_prefix() {
        assert!(parse_spotify_uri("notspotify:track:abc").is_err());
    }

    #[test]
    fn rejects_too_few_parts() {
        assert!(parse_spotify_uri("spotify:track").is_err());
    }

    #[test]
    fn rejects_empty_string() {
        assert!(parse_spotify_uri("").is_err());
    }
}
