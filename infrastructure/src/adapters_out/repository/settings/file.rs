use domain::ports::{
    ports_in::settings::models::{FilterActionView, FilterTargetView},
    ports_out::settings::SettingsFilePort,
};

use crate::adapters_out::repository::settings::{
    dto::settings_dto::SettingsFileDto,
    mapper::settings_mapper::{file_dto_to_view, view_to_file_dto},
};

/// Reads and writes `settings.json` next to the binary (fallback: cwd).
pub struct SettingsFileAdapter;

impl SettingsFileAdapter {
    pub fn new() -> Self {
        Self
    }

    fn path() -> std::path::PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("settings.json")))
            .unwrap_or_else(|| std::path::PathBuf::from("settings.json"))
    }
}

impl SettingsFilePort for SettingsFileAdapter {
    fn load(&self) -> (FilterActionView, FilterTargetView) {
        let path = Self::path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .expect("failed to read settings.json");
            let dto: SettingsFileDto =
                serde_json::from_str(&content).expect("failed to parse settings.json");
            file_dto_to_view(dto)
        } else {
            let dto = SettingsFileDto::default();
            let content = serde_json::to_string_pretty(&dto)
                .expect("failed to serialize settings");
            std::fs::write(&path, content).expect("failed to write settings.json");
            file_dto_to_view(dto)
        }
    }

    fn save(&self, action: &FilterActionView, target: &FilterTargetView) {
        let path = Self::path();
        let dto = view_to_file_dto(action, target);
        let content = serde_json::to_string_pretty(&dto)
            .expect("failed to serialize settings");
        std::fs::write(&path, content).expect("failed to write settings.json");
    }
}
