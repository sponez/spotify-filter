use std::sync::{Arc, Mutex};

use infrastructure::adapters_in::{hotkeys::HotkeyAdapter, tray::TrayAdapter};
use slint::{CloseRequestResponse, ComponentHandle, Timer, TimerMode};

use crate::{AppStateEnum, AppWindow, FilterActionEnum};

use domain::ports::ports_in::spotify::{
    spotify_facade::SpotifyFacade,
    usecases::{
        sign_in::SignInUseCase,
        sign_out::SignOutUseCase,
    },
};

fn filter_action_to_int(a: FilterActionEnum) -> i32 {
    match a {
        FilterActionEnum::None => 0,
        FilterActionEnum::AddToPlaylist => 1,
        FilterActionEnum::MoveToPlaylist => 2,
    }
}

fn int_to_filter_action(n: i32) -> FilterActionEnum {
    match n {
        1 => FilterActionEnum::AddToPlaylist,
        2 => FilterActionEnum::MoveToPlaylist,
        _ => FilterActionEnum::None,
    }
}

pub struct UiWindow {
    window: AppWindow,
    timer: slint::Timer,
}

impl UiWindow {
    fn setup_close_handler(&self) {
        let window_weak = self.window.as_weak();
        self.window.window().on_close_requested(move || {
            if let Some(w) = window_weak.upgrade() {
                let state = w.get_state();
                if state == AppStateEnum::SignedIn || state == AppStateEnum::Settings {
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

    fn setup_open_settings_callback(&self, current: Arc<Mutex<(i32, i32, i32)>>) {
        let w = self.window.as_weak();
        self.window.on_open_settings(move || {
            if let Some(w) = w.upgrade() {
                let (a, t, pi) = *current.lock().unwrap();
                w.set_filter_action(int_to_filter_action(a));
                w.set_filter_target_type(t);
                w.set_filter_playlist_index(pi);
                w.set_state(AppStateEnum::Settings);
            }
        });
    }

    fn setup_save_settings_callback(
        &self,
        on_save: Box<dyn Fn(i32, i32, i32) + 'static>,
    ) {
        let w = self.window.as_weak();
        self.window.on_save_settings(move || {
            if let Some(w) = w.upgrade() {
                on_save(
                    filter_action_to_int(w.get_filter_action()),
                    w.get_filter_target_type(),
                    w.get_filter_playlist_index(),
                );
            }
        });
    }

    pub fn create_and_set_up_callbacks(
        sign_in: Arc<dyn SignInUseCase>,
        sign_out: Arc<dyn SignOutUseCase>,
        current_settings: Arc<Mutex<(i32, i32, i32)>>,
        on_save: Box<dyn Fn(i32, i32, i32) + 'static>,
    ) -> Self {
        let window = AppWindow::new().expect("Failed to create main window");
        let ui_window = Self {
            window,
            timer: Timer::default(),
        };

        ui_window.setup_close_handler();
        ui_window.setup_sign_in_callback(sign_in);
        ui_window.setup_sign_out_callback(sign_out);
        ui_window.setup_open_settings_callback(current_settings);
        ui_window.setup_save_settings_callback(on_save);

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
