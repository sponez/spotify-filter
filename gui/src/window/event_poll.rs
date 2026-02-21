use std::sync::Arc;

use slint::ComponentHandle;

use domain::ports::ports_in::spotify::spotify_facade::SpotifyFacade;
use infrastructure::adapters_in::{hotkeys::HotkeyEventListener, tray::TrayEventListener};

use crate::{AppStateEnum, AppWindow};

pub fn start_event_poll(
    window: &AppWindow,
    timer: &slint::Timer,
    tray: Arc<TrayEventListener>,
    hotkeys: Arc<HotkeyEventListener>,
    spotify_facade: Arc<SpotifyFacade>,
) {
    let window_weak = window.as_weak();

    timer.start(
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
