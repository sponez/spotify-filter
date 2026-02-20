use serde::{Deserialize, Serialize};

use crate::settings::models::filter_action::{AdditionalFilterAction, PlaylistTarget};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub filter_action: AdditionalFilterAction,
    /// Present only when filter_action is AddToPlaylist or MoveToPlaylist.
    pub filter_target: Option<PlaylistTarget>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            filter_action: AdditionalFilterAction::None,
            filter_target: None,
        }
    }
}

impl Settings {
    /// Load from `settings.json` next to the binary (fallback: cwd).
    /// Creates the file with defaults if absent.
    pub fn load() -> Self {
        let path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("settings.json")))
            .unwrap_or_else(|| std::path::PathBuf::from("settings.json"));

        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .expect("failed to read settings.json");
            serde_json::from_str(&content).expect("failed to parse settings.json")
        } else {
            let settings = Settings::default();
            settings.save_to(&path);
            settings
        }
    }

    /// Persist current settings back to `settings.json`.
    pub fn save(&self) {
        let path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("settings.json")))
            .unwrap_or_else(|| std::path::PathBuf::from("settings.json"));
        self.save_to(&path);
    }

    fn save_to(&self, path: &std::path::Path) {
        let content = serde_json::to_string_pretty(self).expect("failed to serialize settings");
        std::fs::write(path, content).expect("failed to write settings.json");
    }
}
