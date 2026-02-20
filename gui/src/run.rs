use std::sync::{Arc, Mutex};

use domain::ports::ports_in::spotify::spotify_facade::SpotifyFacade;
use slint::run_event_loop_until_quit;

use infrastructure::adapters_in::{hotkeys::HotkeyAdapter, tray::TrayAdapter};

use crate::window::UiWindow;

pub fn run(
    tray: TrayAdapter,
    hotkeys: HotkeyAdapter,
    spotify_facade: SpotifyFacade,
    current_settings: Arc<Mutex<(i32, i32, i32)>>,
    on_save: Box<dyn Fn(i32, i32, i32) + 'static>,
) -> Result<(), slint::PlatformError> {
    let window = UiWindow::create_and_set_up_callbacks(
        Arc::clone(&spotify_facade.sign_in),
        Arc::clone(&spotify_facade.sign_out),
        current_settings,
        on_save,
    );
    let tray = Arc::new(tray);
    let hotkeys = Arc::new(hotkeys);
    let spotify_facade = Arc::new(spotify_facade);

    window.start_event_poll(tray, hotkeys, spotify_facade);
    window.show().expect("Window cannot be shown.");
    run_event_loop_until_quit()
}
