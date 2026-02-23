use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, Sender},
};

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
            let _ = self.try_sign_in.try_sign_in();
        }
    }

    pub fn run(self) {
        while let Ok(request) = self.rx.recv() {
            let response = match request {
                AppRequest::SignIn => {
                    let result = self.sign_in.sign_in();
                    if result.is_ok() {
                        self.authorized.store(true, Ordering::Relaxed);
                    }
                    AppResponse::SignInCompleted(result)
                }
                AppRequest::SignOut => {
                    let result = self.sign_out.sign_out();
                    if result.is_ok() {
                        self.authorized.store(false, Ordering::Relaxed);
                    }
                    AppResponse::SignOutCompleted(result)
                }
                AppRequest::FilterTrack => {
                    self.refresh_token_if_needed();
                    AppResponse::FilterTrackCompleted(self.filter_track.filter_current_track())
                }
                AppRequest::PassTrack => {
                    self.refresh_token_if_needed();
                    AppResponse::PassTrackCompleted(self.pass_track.pass_current_track())
                }
                AppRequest::GetSettings => {
                    AppResponse::SettingsLoaded(self.get_settings.get_settings())
                }
                AppRequest::GetPlaylists => {
                    self.refresh_token_if_needed();
                    AppResponse::PlaylistsLoaded(self.get_playlists.get_playlists())
                }
                AppRequest::SaveSettings(command) => {
                    AppResponse::SettingsSaved(self.save_settings.save_settings(command))
                }
                AppRequest::ShowWindow => AppResponse::ShowWindow,
                AppRequest::Quit => {
                    let _ = self.tx.send(AppResponse::Quit);
                    break;
                }
            };
            let _ = self.tx.send(response);
        }
    }
}
