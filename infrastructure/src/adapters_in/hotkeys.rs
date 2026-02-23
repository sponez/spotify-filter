use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
};
use tracing::{debug, info, warn};

use domain::ports::ports_in::events::AppRequest;
use global_hotkey::{ GlobalHotKeyEvent, HotKeyState };

pub struct HotkeyEventListener {
    filter_id: u32,
    pass_id: u32,
}

impl HotkeyEventListener {
    pub fn new(filter_id: u32, pass_id: u32) -> Self {
        Self {
            filter_id,
            pass_id,
        }
    }

    pub fn start_polling(self: Arc<Self>, tx: Sender<AppRequest>, authorized: Arc<AtomicBool>) {
        info!("Starting hotkey polling thread");
        std::thread::spawn(move || loop {
            self.poll(&tx, &authorized);
            std::thread::sleep(std::time::Duration::from_millis(50));
        });
    }

    fn poll(&self, tx: &Sender<AppRequest>, authorized: &Arc<AtomicBool>) {
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state != HotKeyState::Pressed {
                continue;
            }
            if !authorized.load(Ordering::Relaxed) {
                warn!("Hotkey pressed while user is unauthorized");
                continue;
            }
            if event.id == self.filter_id {
                debug!("Filter hotkey pressed");
                let _ = tx.send(AppRequest::FilterTrack);
            } else if event.id == self.pass_id {
                debug!("Pass hotkey pressed");
                let _ = tx.send(AppRequest::PassTrack);
            }
        }
    }
}
