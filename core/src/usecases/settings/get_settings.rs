use std::sync::Arc;

use crate::{
    errors::errors::AppResult,
    ports::{
        ports_in::settings::{
            models::{PlaylistItemView, SettingsView},
            usecases::get_settings::GetSettingsUseCase,
        },
        ports_out::{
            client::spotify_api::SpotifyApiClient,
            notification::ErrorNotification,
            repository::settings::{SettingsCache, SettingsStore},
        },
    },
};

pub struct GetSettingsInteractor {
    cache: Arc<dyn SettingsCache>,
    file: Arc<dyn SettingsStore>,
    api_client: Arc<dyn SpotifyApiClient>,
    notifier: Arc<dyn ErrorNotification>,
}

impl GetSettingsInteractor {
    pub fn new(
        cache: Arc<dyn SettingsCache>,
        file: Arc<dyn SettingsStore>,
        api_client: Arc<dyn SpotifyApiClient>,
        notifier: Arc<dyn ErrorNotification>,
    ) -> Self {
        Self { cache, file, api_client, notifier }
    }
}

impl GetSettingsUseCase for GetSettingsInteractor {
    fn get_settings(&self) -> AppResult<SettingsView> {
        let current_cache = self.cache.load();
        let (filter_action, filter_target) = match current_cache {
            Some(pair) => pair,
            None => {
                let pair = self.file.load().map_err(|e| {
                    self.notifier.notify(&e.to_string());
                    e
                })?;
                self.cache.store(&pair.0, &pair.1);
                pair
            }
        };

        let playlists = self.api_client.get_my_playlists()
            .unwrap_or_default()
            .into_iter()
            .map(|p| PlaylistItemView { id: p.id, name: p.name })
            .collect();

        Ok(SettingsView { filter_action, filter_target, playlists })
    }
}
