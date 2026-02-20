use std::sync::Arc;

use slint::{ComponentHandle, run_event_loop_until_quit};

use domain::ports::ports_in::spotify::{PlayerUseCase, SignInUseCase, SignOutUseCase};
use infrastructure::adapters_in::{hotkeys::HotkeyAdapter, tray::TrayAdapter};

use crate::AppWindow;
use crate::event_loop::start_event_poll;
use crate::window::{setup_close_handler, setup_sign_in_callback, setup_sign_out_callback};

pub fn run(
    tray: TrayAdapter,
    hotkeys: HotkeyAdapter,
    player: Arc<dyn PlayerUseCase>,
    sign_in: Arc<dyn SignInUseCase>,
    sign_out: Arc<dyn SignOutUseCase>,
) -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;

    setup_close_handler(&window);
    setup_sign_in_callback(&window, Arc::clone(&sign_in));
    setup_sign_out_callback(&window, Arc::clone(&sign_out));

    let tray = Arc::new(tray);
    let hotkeys = Arc::new(hotkeys);
    let _timer = start_event_poll(&window, tray, hotkeys, player, sign_out);

    window.show()?;
    run_event_loop_until_quit()
}
