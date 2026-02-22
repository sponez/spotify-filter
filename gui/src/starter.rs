use std::sync::mpsc::{Receiver, Sender};

use domain::ports::ports_in::events::{AppRequest, AppResponse};
use slint::run_event_loop_until_quit;

use crate::window::UiWindow;

pub fn run(
    tx: Sender<AppRequest>,
    rx: Receiver<AppResponse>,
) -> Result<(), slint::PlatformError> {
    let window = UiWindow::create_and_set_up_callbacks(tx.clone());

    window.start_event_poll(rx);
    window.show()?;
    run_event_loop_until_quit()
}
