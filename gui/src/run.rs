use std::sync::Arc;

use domain::ports::ports_in::spotify::spotify_facade::{SpotifyFacade};
use slint::{ComponentHandle, run_event_loop_until_quit};

use infrastructure::adapters_in::{hotkeys::HotkeyAdapter, tray::TrayAdapter};

use crate::AppWindow;
use crate::event_loop::start_event_poll;
use crate::window::{setup_close_handler, setup_sign_in_callback, setup_sign_out_callback};

pub fn run(
    tray: TrayAdapter,
    hotkeys: HotkeyAdapter,
    spotify_facade: SpotifyFacade,
) -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;

    setup_close_handler(&window);
    setup_sign_in_callback(&window, Arc::clone(&spotify_facade.sign_in));
    setup_sign_out_callback(&window, Arc::clone(&spotify_facade.sign_out));

    let tray = Arc::new(tray);
    let hotkeys = Arc::new(hotkeys);
    let spotify_facade = Arc::new(spotify_facade);
    let _timer = start_event_poll(&window, tray, hotkeys, spotify_facade);

    window.show()?;
    run_event_loop_until_quit()
}
