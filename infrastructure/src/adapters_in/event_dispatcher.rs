use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, Sender},
};

use domain::ports::ports_in::{
    events::{AppRequest, AppResponse},
    settings::usecases::{
        get_settings::GetSettingsUseCase,
        save_settings::SaveSettingsUseCase,
    },
    spotify::usecases::{
        filter_track::FilterTrackUseCase,
        pass_track::PassTrackUseCase,
        sign_in::SignInUseCase,
        sign_out::SignOutUseCase,
    },
};

pub struct EventDispatcher {
    rx: Receiver<AppRequest>,
    tx: Sender<AppResponse>,
    authorized: Arc<AtomicBool>,
    sign_in: Arc<dyn SignInUseCase>,
    sign_out: Arc<dyn SignOutUseCase>,
    filter_track: Arc<dyn FilterTrackUseCase>,
    pass_track: Arc<dyn PassTrackUseCase>,
    get_settings: Arc<dyn GetSettingsUseCase>,
    save_settings: Arc<dyn SaveSettingsUseCase>,
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
        save_settings: Arc<dyn SaveSettingsUseCase>,
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
            save_settings,
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
                    AppResponse::FilterTrackCompleted(self.filter_track.filter_current_track())
                }
                AppRequest::PassTrack => {
                    AppResponse::PassTrackCompleted(self.pass_track.pass_current_track())
                }
                AppRequest::GetSettings => {
                    AppResponse::SettingsLoaded(self.get_settings.get_settings())
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
