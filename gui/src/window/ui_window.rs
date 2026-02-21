use std::sync::Arc;

use domain::ports::ports_in::{
    settings::settings_facade::SettingsFacade,
    spotify::usecases::{sign_in::SignInUseCase, sign_out::SignOutUseCase},
};
use infrastructure::adapters_in::{hotkeys::HotkeyEventListener, tray::TrayEventListener};

use slint::ComponentHandle;

use crate::{AppStateEnum, AppWindow};
use crate::window::{
    callbacks::{
        auth::{setup_close_handler, setup_sign_in_callback, setup_sign_out_callback},
        settings::{setup_open_settings_callback, setup_save_settings_callback},
    }
};

use domain::ports::ports_in::spotify::spotify_facade::SpotifyFacade;

pub struct UiWindow {
    pub(super) window: AppWindow,
    pub(super) timer: slint::Timer,
}

impl UiWindow {
    pub fn create_and_set_up_callbacks(
        sign_in: Arc<dyn SignInUseCase>,
        sign_out: Arc<dyn SignOutUseCase>,
        settings_facade: Arc<SettingsFacade>,
    ) -> Self {
        let window = AppWindow::new().expect("Failed to create main window");
        let ui_window = Self { window, timer: slint::Timer::default() };

        setup_close_handler(&ui_window.window);
        setup_sign_in_callback(&ui_window.window, sign_in);
        setup_sign_out_callback(&ui_window.window, sign_out);
        setup_open_settings_callback(&ui_window.window, Arc::clone(&settings_facade.get));
        setup_save_settings_callback(&ui_window.window, Arc::clone(&settings_facade.save));

        ui_window
    }

    pub fn show(&self) -> Result<(), slint::PlatformError> {
        self.window.show()
    }

    pub fn start_event_poll(
        &self,
        tray: Arc<TrayEventListener>,
        hotkeys: Arc<HotkeyEventListener>,
        spotify_facade: Arc<SpotifyFacade>,
    ) {
        let window_weak = self.window.as_weak();

        self.timer.start(
            slint::TimerMode::Repeated,
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
