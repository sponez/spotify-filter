use crate::ports::ports_in::settings::models::{FilterActionView, FilterTargetView};

/// Read settings from the cache layer (fast, in-memory).
/// Returns `None` if the cache is cold (not yet populated).
pub trait SettingsCache: Send + Sync {
    fn load(&self) -> Option<(FilterActionView, FilterTargetView)>;
    fn store(&self, action: &FilterActionView, target: &FilterTargetView);
}

/// Persist / restore settings from durable storage (file, registry, …).
pub trait SettingsStore: Send + Sync {
    fn load(&self) -> (FilterActionView, FilterTargetView);
    fn save(&self, action: &FilterActionView, target: &FilterTargetView);
}
