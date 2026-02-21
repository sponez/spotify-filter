use domain::ports::{
    ports_in::settings::models::{FilterActionView, FilterTargetView},
    ports_out::repository::settings::{SettingsStore, SettingsStoreError},
};

use crate::adapters_out::repository::settings::{
    dto::settings_dto::SettingsFileDto,
    mapper::settings_mapper::{file_dto_to_view, view_to_file_dto},
};

/// Reads and writes `settings.json` next to the binary (fallback: cwd).
pub struct JsonFileSettingsStore;

impl JsonFileSettingsStore {
    pub fn new() -> Self {
        Self
    }

    fn path() -> std::path::PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("settings.json")))
            .unwrap_or_else(|| std::path::PathBuf::from("settings.json"))
    }

    fn default_settings() -> (FilterActionView, FilterTargetView) {
        file_dto_to_view(SettingsFileDto::default())
    }
}

impl SettingsStore for JsonFileSettingsStore {
    fn load(&self) -> Result<(FilterActionView, FilterTargetView), SettingsStoreError> {
        let path = Self::path();

        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| SettingsStoreError::ReadFailed(e.into()))?;
            let dto: SettingsFileDto = serde_json::from_str(&content)
                .map_err(|e| SettingsStoreError::ParseFailed(e.into()))?;
            Ok(file_dto_to_view(dto))
        } else {
            Ok(Self::default_settings())
        }
    }

    fn save(&self, action: &FilterActionView, target: &FilterTargetView) -> Result<(), SettingsStoreError> {
        let path = Self::path();
        let tmp = path.with_extension("tmp");

        let dto = view_to_file_dto(action, target);
        let content = serde_json::to_string_pretty(&dto)
            .map_err(|e| SettingsStoreError::WriteFailed(e.into()))?;

        std::fs::write(&tmp, content)
            .map_err(|e| SettingsStoreError::WriteFailed(e.into()))?;
        std::fs::rename(&tmp, &path)
            .map_err(|e| SettingsStoreError::WriteFailed(e.into()))?;

        Ok(())
    }
}
