use std::sync::Arc;

use domain::ports::ports_in::{
    settings::settings_facade::SettingsFacade,
    spotify::spotify_facade::SpotifyFacade,
};
use slint::run_event_loop_until_quit;

use infrastructure::adapters_in::{hotkeys::HotkeyEventListener, tray::TrayEventListener};

use crate::window::UiWindow;

pub fn run(
    tray: TrayEventListener,
    hotkeys: HotkeyEventListener,
    spotify_facade: SpotifyFacade,
    settings_facade: SettingsFacade,
) -> Result<(), slint::PlatformError> {
    let settings_facade = Arc::new(settings_facade);
    let window = UiWindow::create_and_set_up_callbacks(
        Arc::clone(&spotify_facade.sign_in),
        Arc::clone(&spotify_facade.sign_out),
        settings_facade,
    );
    let tray = Arc::new(tray);
    let hotkeys = Arc::new(hotkeys);
    let spotify_facade = Arc::new(spotify_facade);

    window.start_event_poll(tray, hotkeys, spotify_facade);
    window.show()?;
    run_event_loop_until_quit()
}
