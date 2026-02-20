use std::sync::Arc;

use infrastructure::adapters_in::{hotkeys::HotkeyAdapter, tray::TrayAdapter};
use slint::{CloseRequestResponse, ComponentHandle, Timer, TimerMode};

use crate::{AppStateEnum, AppWindow};

use domain::ports::ports_in::spotify::{
    spotify_facade::SpotifyFacade,
    usecases::{
        sign_in::SignInUseCase,
        sign_out::SignOutUseCase
    }
};

pub struct UiWindow {
    window: AppWindow,
    timer: slint::Timer,
}

impl UiWindow {
    fn setup_close_handler(&self) {
        let window_weak = self.window.as_weak();
        self.window.window().on_close_requested(move || {
            if let Some(w) = window_weak.upgrade() {
                if w.get_state() == AppStateEnum::SignedIn {
                    w.window().hide().ok();
                    return CloseRequestResponse::KeepWindowShown;
                }
            }
            slint::quit_event_loop().ok();
            CloseRequestResponse::KeepWindowShown
        });
    }

    fn setup_sign_in_callback(&self, auth: Arc<dyn SignInUseCase>) {
        let window_weak = self.window.as_weak();
        self.window.on_sign_in(move || {
            if let Some(w) = window_weak.upgrade() {
                w.set_state(AppStateEnum::AwaitLogin);
                auth.sign_in();
                // TODO: listen for auth completion and transition to SignedIn
                w.set_state(AppStateEnum::SignedIn);
            }
        });
    }

    fn setup_sign_out_callback(&self, auth: Arc<dyn SignOutUseCase>) {
        let window_weak = self.window.as_weak();
        self.window.on_sign_out(move || {
            if let Some(w) = window_weak.upgrade() {
                auth.sign_out();
                w.set_state(AppStateEnum::Login);
                w.window().show().ok();
            }
        });
    }

    pub fn create_and_set_up_callbacks(
        sign_in: Arc<dyn SignInUseCase>,
        sign_out: Arc<dyn SignOutUseCase>,
    ) -> Self {
        let window = AppWindow::new().expect("Failed to create main window");
        let ui_window = Self {
            window,
            timer: Timer::default(),
        };

        ui_window.setup_close_handler();
        ui_window.setup_sign_in_callback(sign_in);
        ui_window.setup_sign_out_callback(sign_out);

        ui_window
    }

    pub fn show(&self) -> Result<(), slint::PlatformError> {
        self.window.show()
    }

    pub fn start_event_poll(
        &self,
        tray: Arc<TrayAdapter>,
        hotkeys: Arc<HotkeyAdapter>,
        spotify_facade: Arc<SpotifyFacade>,
    ) {
        let window_weak = self.window.as_weak();

        self.timer.start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(100),
            move || {
                let Some(w) = window_weak.upgrade() else { return };
                let w2 = window_weak.clone();
                let sign_out_clone = Arc::clone(&spotify_facade.sign_out);
                let pass_track_clone = Arc::clone(&spotify_facade.pass_track);
                let filter_track_clone = Arc::clone(&spotify_facade.filter_track);

                tray.poll(
                    move || {
                        w.window().show().ok();
                    },
                    move || {
                        sign_out_clone.sign_out();
                        if let Some(win) = w2.upgrade() {
                            win.set_state(AppStateEnum::Login);
                            win.window().show().ok();
                        }
                    },
                    || {
                        slint::quit_event_loop().ok();
                    },
                );

                hotkeys.poll(
                    move || {
                        filter_track_clone.filter_current_track();
                    },
                    move || {
                        pass_track_clone.pass_current_track();
                    },
                );
            },
        );
    }
}
