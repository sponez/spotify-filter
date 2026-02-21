use thiserror::Error;

use crate::ports::ports_in::settings::models::{FilterActionView, FilterTargetView};

#[derive(Debug, Error)]
pub enum SettingsStoreError {
    #[error("Failed to read settings")]
    ReadFailed(#[source] anyhow::Error),

    #[error("Failed to write settings")]
    WriteFailed(#[source] anyhow::Error),

    #[error("Failed to parse settings")]
    ParseFailed(#[source] anyhow::Error),
}

/// Read settings from the cache layer (fast, in-memory).
/// Returns `None` if the cache is cold (not yet populated).
pub trait SettingsCache: Send + Sync {
    fn load(&self) -> Option<(FilterActionView, FilterTargetView)>;
    fn store(&self, action: &FilterActionView, target: &FilterTargetView);
}

/// Persist / restore settings from durable storage (file, registry, …).
pub trait SettingsStore: Send + Sync {
    fn load(&self) -> Result<(FilterActionView, FilterTargetView), SettingsStoreError>;
    fn save(&self, action: &FilterActionView, target: &FilterTargetView) -> Result<(), SettingsStoreError>;
}
