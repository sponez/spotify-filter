use std::sync::mpsc::{Receiver, Sender};

use domain::ports::ports_in::events::{AppRequest, AppResponse};

use slint::ComponentHandle;
use tracing::{debug, error, info};

use crate::{AppStateEnum, AppWindow};
use crate::window::{
    callbacks::{
        auth::{setup_close_handler, setup_sign_in_callback, setup_sign_out_callback},
        settings::{setup_open_settings_callback, setup_save_settings_callback},
    },
    mapper::settings_mapper::{apply_playlists_to_window, apply_settings_view_to_window},
};

pub struct UiWindow {
    pub(super) window: AppWindow,
    pub(super) timer: slint::Timer,
}

impl UiWindow {
    pub fn create_and_set_up_callbacks(tx: Sender<AppRequest>) -> Self {
        info!("Creating main window");
        let window = AppWindow::new().expect("Failed to create main window");
        let ui_window = Self { window, timer: slint::Timer::default() };

        setup_close_handler(&ui_window.window);
        setup_sign_in_callback(&ui_window.window, tx.clone());
        setup_sign_out_callback(&ui_window.window, tx.clone());
        setup_open_settings_callback(&ui_window.window, tx.clone());
        setup_save_settings_callback(&ui_window.window, tx);

        ui_window
    }

    pub fn show(&self) -> Result<(), slint::PlatformError> {
        info!("Showing main window");
        self.window.show()
    }

    pub fn set_state(&self, state: AppStateEnum) {
        self.window.set_state(state);
    }

    pub fn set_hotkeys(&self, filter_hotkey: &str, pass_hotkey: &str) {
        self.window.set_filter_hotkey(slint::SharedString::from(filter_hotkey));
        self.window.set_pass_hotkey(slint::SharedString::from(pass_hotkey));
    }

    pub fn start_event_poll(&self, tx: Sender<AppRequest>, rx: Receiver<AppResponse>) {
        info!("Starting GUI response polling timer");
        let window_weak = self.window.as_weak();

        self.timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(100),
            move || {
                while let Ok(response) = rx.try_recv() {
                    let Some(w) = window_weak.upgrade() else { continue };
                    match response {
                        AppResponse::SignInCompleted(result) => {
                            if result.is_ok() {
                                info!("Sign-in completed successfully");
                                w.set_state(AppStateEnum::SignedIn);
                                w.window().hide().ok();
                            } else {
                                error!("Sign-in completed with error");
                                w.set_state(AppStateEnum::Login);
                            }
                        }
                        AppResponse::SignOutCompleted(result) => {
                            if result.is_ok() {
                                info!("Sign-out completed successfully");
                                w.set_state(AppStateEnum::Login);
                                w.window().show().ok();
                            }
                        }
                        AppResponse::FilterTrackCompleted(result) => {
                            if let Err(e) = result {
                                error!(error = %e, "Filter track request failed");
                            } else {
                                debug!("Filter track request completed");
                            }
                        }
                        AppResponse::PassTrackCompleted(result) => {
                            if let Err(e) = result {
                                error!(error = %e, "Pass track request failed");
                            } else {
                                debug!("Pass track request completed");
                            }
                        }
                        AppResponse::SettingsLoaded(result) => {
                            if let Ok(view) = result {
                                info!("Settings loaded into GUI");
                                apply_settings_view_to_window(&w, view);
                                w.set_state(AppStateEnum::Settings);
                                let _ = tx.send(AppRequest::GetPlaylists);
                            }
                        }
                        AppResponse::PlaylistsLoaded(result) => {
                            match result {
                                Ok(playlists) => {
                                    info!(count = playlists.len(), "Playlists loaded into GUI");
                                    apply_playlists_to_window(&w, playlists);
                                }
                                Err(e) => {
                                    error!(error = %e, "Playlists load failed");
                                }
                            }
                        }
                        AppResponse::SettingsSaved(result) => {
                            if result.is_ok() {
                                info!("Settings saved");
                                w.set_state(AppStateEnum::SignedIn);
                            }
                        }
                        AppResponse::ShowWindow => {
                            info!("Show window event received");
                            w.window().show().ok();
                        }
                        AppResponse::Quit => {
                            info!("Quit event received");
                            slint::quit_event_loop().ok();
                        }
                    }
                }
            },
        );
    }
}
