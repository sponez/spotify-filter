use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, Sender},
};
use tracing::{debug, error, info, warn};

use domain::ports::ports_in::{
    events::{AppRequest, AppResponse},
    settings::usecases::{
        get_playlists::GetPlaylistsUseCase,
        get_settings::GetSettingsUseCase,
        save_settings::SaveSettingsUseCase,
    },
    spotify::usecases::{
        filter_track::FilterTrackUseCase,
        pass_track::PassTrackUseCase,
        sign_in::SignInUseCase,
        sign_out::SignOutUseCase,
        try_sign_in::TrySignInUseCase,
    },
};
use domain::ports::ports_out::repository::token::TokenCache;

pub struct EventDispatcher {
    rx: Receiver<AppRequest>,
    tx: Sender<AppResponse>,
    authorized: Arc<AtomicBool>,
    sign_in: Arc<dyn SignInUseCase>,
    sign_out: Arc<dyn SignOutUseCase>,
    filter_track: Arc<dyn FilterTrackUseCase>,
    pass_track: Arc<dyn PassTrackUseCase>,
    get_settings: Arc<dyn GetSettingsUseCase>,
    get_playlists: Arc<dyn GetPlaylistsUseCase>,
    save_settings: Arc<dyn SaveSettingsUseCase>,
    try_sign_in: Arc<dyn TrySignInUseCase>,
    token_cache: Arc<dyn TokenCache>,
}

impl EventDispatcher {
    pub fn new(
        rx: Receiver<AppRequest>,
        tx: Sender<AppResponse>,
        authorized: Arc<AtomicBool>,
        sign_in: Arc<dyn SignInUseCase>,
        sign_out: Arc<dyn SignOutUseCase>,
        filter_track: Arc<dyn FilterTrackUseCase>,
        pass_track: Arc<dyn PassTrackUseCase>,
        get_settings: Arc<dyn GetSettingsUseCase>,
        get_playlists: Arc<dyn GetPlaylistsUseCase>,
        save_settings: Arc<dyn SaveSettingsUseCase>,
        try_sign_in: Arc<dyn TrySignInUseCase>,
        token_cache: Arc<dyn TokenCache>,
    ) -> Self {
        Self {
            rx,
            tx,
            authorized,
            sign_in,
            sign_out,
            filter_track,
            pass_track,
            get_settings,
            get_playlists,
            save_settings,
            try_sign_in,
            token_cache,
        }
    }

    fn refresh_token_if_needed(&self) {
        if self.token_cache.is_expiring_soon() {
            info!("Access token expiring soon, trying refresh");
            match self.try_sign_in.try_sign_in() {
                Ok(true) => info!("Token refresh succeeded"),
                Ok(false) => warn!("Token refresh skipped: no refresh token"),
                Err(e) => error!(error = %e, "Token refresh failed"),
            }
        }
    }

    pub fn run(self) {
        info!("Event dispatcher loop started");
        while let Ok(request) = self.rx.recv() {
            let request_name = match &request {
                AppRequest::SignIn => "SignIn",
                AppRequest::SignOut => "SignOut",
                AppRequest::FilterTrack => "FilterTrack",
                AppRequest::PassTrack => "PassTrack",
                AppRequest::GetSettings => "GetSettings",
                AppRequest::GetPlaylists => "GetPlaylists",
                AppRequest::SaveSettings(_) => "SaveSettings",
                AppRequest::ShowWindow => "ShowWindow",
                AppRequest::Quit => "Quit",
            };
            debug!(request = request_name, "Received app request");
            let response = match request {
                AppRequest::SignIn => {
                    let result = self.sign_in.sign_in();
                    if result.is_ok() {
                        self.authorized.store(true, Ordering::Relaxed);
                        info!("Authorization state set to signed-in");
                    } else if let Err(ref e) = result {
                        error!(error = %e, "Sign-in failed");
                    }
                    AppResponse::SignInCompleted(result)
                }
                AppRequest::SignOut => {
                    let result = self.sign_out.sign_out();
                    if result.is_ok() {
                        self.authorized.store(false, Ordering::Relaxed);
                        info!("Authorization state set to signed-out");
                    } else if let Err(ref e) = result {
                        error!(error = %e, "Sign-out failed");
                    }
                    AppResponse::SignOutCompleted(result)
                }
                AppRequest::FilterTrack => {
                    self.refresh_token_if_needed();
                    let result = self.filter_track.filter_current_track();
                    if let Err(ref e) = result {
                        error!(error = %e, "Filter track command failed");
                    }
                    AppResponse::FilterTrackCompleted(result)
                }
                AppRequest::PassTrack => {
                    self.refresh_token_if_needed();
                    let result = self.pass_track.pass_current_track();
                    if let Err(ref e) = result {
                        error!(error = %e, "Pass track command failed");
                    }
                    AppResponse::PassTrackCompleted(result)
                }
                AppRequest::GetSettings => {
                    let result = self.get_settings.get_settings();
                    if let Err(ref e) = result {
                        error!(error = %e, "Get settings command failed");
                    }
                    AppResponse::SettingsLoaded(result)
                }
                AppRequest::GetPlaylists => {
                    self.refresh_token_if_needed();
                    let result = self.get_playlists.get_playlists();
                    if let Err(ref e) = result {
                        error!(error = %e, "Get playlists command failed");
                    }
                    AppResponse::PlaylistsLoaded(result)
                }
                AppRequest::SaveSettings(command) => {
                    let result = self.save_settings.save_settings(command);
                    if let Err(ref e) = result {
                        error!(error = %e, "Save settings command failed");
                    }
                    AppResponse::SettingsSaved(result)
                }
                AppRequest::ShowWindow => AppResponse::ShowWindow,
                AppRequest::Quit => {
                    info!("Quit requested");
                    let _ = self.tx.send(AppResponse::Quit);
                    break;
                }
            };
            let _ = self.tx.send(response);
        }
        warn!("Event dispatcher loop ended");
    }
}
