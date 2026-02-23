use std::sync::mpsc::{Receiver, Sender};

use domain::ports::ports_in::events::{AppRequest, AppResponse};
use slint::run_event_loop_until_quit;

use crate::{AppStateEnum, window::UiWindow};

pub fn run(
    tx: Sender<AppRequest>,
    rx: Receiver<AppResponse>,
    initially_authorized: bool,
    filter_hotkey: &str,
    pass_hotkey: &str,
) -> Result<(), slint::PlatformError> {
    let window = UiWindow::create_and_set_up_callbacks(tx.clone());

    window.set_hotkeys(filter_hotkey, pass_hotkey);
    window.start_event_poll(tx, rx);

    if initially_authorized {
        window.set_state(AppStateEnum::SignedIn);
    } else {
        window.show()?;
    }

    run_event_loop_until_quit()
}
