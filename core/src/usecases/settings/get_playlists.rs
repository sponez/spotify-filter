use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::settings::{
            models::PlaylistItemView,
            usecases::get_playlists::GetPlaylistsUseCase,
        },
        ports_out::{
            client::spotify_api::SpotifyApiClient,
            notification::ErrorNotification,
        },
    },
};

pub struct GetPlaylistsInteractor {
    api_client: Arc<dyn SpotifyApiClient>,
    notifier: Arc<dyn ErrorNotification>,
    cache: Mutex<PlaylistCacheState>,
}

struct PlaylistCacheState {
    loaded_at: Option<Instant>,
    items: Vec<PlaylistItemView>,
}

impl GetPlaylistsInteractor {
    const CACHE_TTL: Duration = Duration::from_secs(300);

    pub fn new(api_client: Arc<dyn SpotifyApiClient>, notifier: Arc<dyn ErrorNotification>) -> Self {
        Self {
            api_client,
            notifier,
            cache: Mutex::new(PlaylistCacheState {
                loaded_at: None,
                items: Vec::new(),
            }),
        }
    }
}

impl GetPlaylistsUseCase for GetPlaylistsInteractor {
    fn get_playlists(&self) -> AppResult<Vec<PlaylistItemView>> {
        let now = Instant::now();
        {
            let cache = match self.cache.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            if let Some(loaded_at) = cache.loaded_at {
                if now.saturating_duration_since(loaded_at) < Self::CACHE_TTL {
                    info!(count = cache.items.len(), "Returning playlists from cache");
                    return Ok(cache.items.clone());
                }
            }
        }

        info!("Loading playlists from Spotify API");
        let fetched = self.api_client
            .get_my_playlists()
            .map(|items| {
                items
                    .into_iter()
                    .map(|p| PlaylistItemView { id: p.id, name: p.name })
                    .collect::<Vec<_>>()
            });

        let fetched = match fetched {
            Ok(items) => items,
            Err(e) => {
                error!(error = %e, "Failed to load playlists");
                self.notifier.notify(&e.to_string());

                let cache = match self.cache.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                if !cache.items.is_empty() {
                    warn!(
                        count = cache.items.len(),
                        "Using stale playlists cache because Spotify request failed"
                    );
                    return Ok(cache.items.clone());
                }
                return Err(e);
            }
        };

        if fetched.is_empty() {
            let cache = match self.cache.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            if !cache.items.is_empty() {
                warn!(
                    count = cache.items.len(),
                    "Using stale playlists cache because Spotify returned empty list"
                );
                return Ok(cache.items.clone());
            }
        } else {
            let mut cache = match self.cache.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            cache.items = fetched.clone();
            cache.loaded_at = Some(now);
            info!(count = cache.items.len(), "Playlists cache updated");
        }

        Ok(fetched)
    }
}
