use std::sync::mpsc::{Receiver, Sender};

use domain::ports::ports_in::events::{AppRequest, AppResponse};

use slint::ComponentHandle;

use crate::{AppStateEnum, AppWindow};
use crate::window::{
    callbacks::{
        auth::{setup_close_handler, setup_sign_in_callback, setup_sign_out_callback},
        settings::{setup_open_settings_callback, setup_save_settings_callback},
    },
    mapper::settings_mapper::apply_settings_view_to_window,
};

pub struct UiWindow {
    pub(super) window: AppWindow,
    pub(super) timer: slint::Timer,
}

impl UiWindow {
    pub fn create_and_set_up_callbacks(tx: Sender<AppRequest>) -> Self {
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
        self.window.show()
    }

    pub fn set_state(&self, state: AppStateEnum) {
        self.window.set_state(state);
    }

    pub fn start_event_poll(&self, rx: Receiver<AppResponse>) {
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
                                w.set_state(AppStateEnum::SignedIn);
                                w.window().hide().ok();
                            } else {
                                w.set_state(AppStateEnum::Login);
                            }
                        }
                        AppResponse::SignOutCompleted(result) => {
                            if result.is_ok() {
                                w.set_state(AppStateEnum::Login);
                                w.window().show().ok();
                            }
                        }
                        AppResponse::FilterTrackCompleted(_) => {}
                        AppResponse::PassTrackCompleted(_) => {}
                        AppResponse::SettingsLoaded(result) => {
                            if let Ok(view) = result {
                                apply_settings_view_to_window(&w, view);
                                w.set_state(AppStateEnum::Settings);
                            }
                        }
                        AppResponse::SettingsSaved(result) => {
                            if result.is_ok() {
                                w.set_state(AppStateEnum::SignedIn);
                            }
                        }
                        AppResponse::ShowWindow => {
                            w.window().show().ok();
                        }
                        AppResponse::Quit => {
                            slint::quit_event_loop().ok();
                        }
                    }
                }
            },
        );
    }
}
