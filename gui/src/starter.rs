use std::sync::{
    Arc,
    atomic::AtomicBool,
    mpsc::{Receiver, Sender},
};

use domain::ports::ports_in::events::{AppRequest, AppResponse};
use infrastructure::adapters_in::{hotkeys::HotkeyEventListener, tray::TrayEventListener};
use slint::run_event_loop_until_quit;

use crate::window::UiWindow;

pub fn run(
    tray: TrayEventListener,
    hotkeys: HotkeyEventListener,
    authorized: Arc<AtomicBool>,
    tx: Sender<AppRequest>,
    rx: Receiver<AppResponse>,
) -> Result<(), slint::PlatformError> {
    let window = UiWindow::create_and_set_up_callbacks(tx.clone());

    let tray = Arc::new(tray);
    let hotkeys = Arc::new(hotkeys);

    window.start_event_poll(tray, hotkeys, authorized, tx, rx);
    window.show()?;
    run_event_loop_until_quit()
}
