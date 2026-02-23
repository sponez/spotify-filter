use crate::{
    errors::errors::AppResult,
    ports::ports_in::settings::models::{PlaylistItemView, SaveSettingsCommand, SettingsView},
};

pub enum AppRequest {
    SignIn,
    SignOut,
    FilterTrack,
    PassTrack,
    GetSettings,
    GetPlaylists,
    SaveSettings(SaveSettingsCommand),
    ShowWindow,
    Quit,
}

pub enum AppResponse {
    SignInCompleted(AppResult<()>),
    SignOutCompleted(AppResult<()>),
    FilterTrackCompleted(AppResult<()>),
    PassTrackCompleted(AppResult<()>),
    SettingsLoaded(AppResult<SettingsView>),
    PlaylistsLoaded(Vec<PlaylistItemView>),
    SettingsSaved(AppResult<()>),
    ShowWindow,
    Quit,
}
