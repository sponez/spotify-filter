use crate::{
    errors::errors::AppResult,
    ports::ports_in::settings::models::{SaveSettingsCommand, SettingsView},
};

pub enum AppRequest {
    SignIn,
    SignOut,
    FilterTrack,
    PassTrack,
    GetSettings,
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
    SettingsSaved(AppResult<()>),
    ShowWindow,
    Quit,
}
