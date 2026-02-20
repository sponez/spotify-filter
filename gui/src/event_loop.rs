use std::sync::Arc;

use slint::{ComponentHandle, Timer, TimerMode};

use domain::ports::ports_in::spotify::{PlayerUseCase, SignOutUseCase};
use infrastructure::adapters_in::{hotkeys::HotkeyAdapter, tray::TrayAdapter};

use crate::{AppStateEnum, AppWindow};

/// Starts the 100ms polling timer that drains tray, menu, and hotkey event
/// channels and dispatches them to the appropriate action handlers.
///
/// Returns the [`Timer`] — the caller must keep it alive.
pub fn start_event_poll(
    window: &AppWindow,
    tray: Arc<TrayAdapter>,
    hotkeys: Arc<HotkeyAdapter>,
    player: Arc<dyn PlayerUseCase>,
    auth: Arc<dyn SignOutUseCase>,
) -> Timer {
    let window_weak = window.as_weak();
    let timer = Timer::default();

    timer.start(
        TimerMode::Repeated,
        std::time::Duration::from_millis(100),
        move || {
            let Some(w) = window_weak.upgrade() else { return };
            let w2 = window_weak.clone();
            let auth_clone = Arc::clone(&auth);
            let player_discard = Arc::clone(&player);
            let player_like = Arc::clone(&player);

            tray.poll(
                move || {
                    w.window().show().ok();
                },
                move || {
                    auth_clone.sign_out();
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
                    player_discard.discard_track();
                },
                move || {
                    player_like.like_and_discard_track();
                },
            );
        },
    );

    timer
}
