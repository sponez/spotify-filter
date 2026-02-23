use std::sync::mpsc::{Receiver, Sender};

use domain::ports::ports_in::events::{AppRequest, AppResponse};
use slint::run_event_loop_until_quit;
use tracing::info;

use crate::{AppStateEnum, window::UiWindow};

pub fn run(
    tx: Sender<AppRequest>,
    rx: Receiver<AppResponse>,
    initially_authorized: bool,
    filter_hotkey: &str,
    pass_hotkey: &str,
) -> Result<(), slint::PlatformError> {
    info!("Starting GUI");
    let window = UiWindow::create_and_set_up_callbacks(tx.clone());

    window.set_hotkeys(filter_hotkey, pass_hotkey);
    window.start_event_poll(tx, rx);

    if initially_authorized {
        info!("GUI starts in SignedIn state");
        window.set_state(AppStateEnum::SignedIn);
    } else {
        info!("GUI starts in Login state");
        window.show()?;
    }

    info!("Entering GUI event loop");
    run_event_loop_until_quit()
}
