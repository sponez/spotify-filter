use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use domain::ports::ports_in::events::AppRequest;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};

pub struct HotkeyEventListener {
    filter_id: u32,
    pass_id: u32,
    throttle: Mutex<HotkeyThrottleState>,
}

struct HotkeyThrottleState {
    last_filter: Option<Instant>,
    last_pass: Option<Instant>,
}

impl HotkeyEventListener {
    const HOTKEY_DEBOUNCE: Duration = Duration::from_millis(500);

    pub fn new(filter_id: u32, pass_id: u32) -> Self {
        Self {
            filter_id,
            pass_id,
            throttle: Mutex::new(HotkeyThrottleState {
                last_filter: None,
                last_pass: None,
            }),
        }
    }

    fn allow_hotkey(now: Instant, last: &mut Option<Instant>) -> bool {
        match last {
            Some(prev) if now.saturating_duration_since(*prev) < Self::HOTKEY_DEBOUNCE => false,
            _ => {
                *last = Some(now);
                true
            }
        }
    }

    pub fn start_polling(self: Arc<Self>, tx: Sender<AppRequest>, authorized: Arc<AtomicBool>) {
        info!("Starting hotkey polling thread");
        std::thread::spawn(move || {
            loop {
                self.poll(&tx, &authorized);
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
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
            let now = Instant::now();
            let mut throttle = match self.throttle.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            if event.id == self.filter_id {
                if !Self::allow_hotkey(now, &mut throttle.last_filter) {
                    debug!("Filter hotkey throttled");
                    continue;
                }
                debug!("Filter hotkey pressed");
                let _ = tx.send(AppRequest::FilterTrack);
            } else if event.id == self.pass_id {
                if !Self::allow_hotkey(now, &mut throttle.last_pass) {
                    debug!("Pass hotkey throttled");
                    continue;
                }
                debug!("Pass hotkey pressed");
                let _ = tx.send(AppRequest::PassTrack);
            }
        }
    }
}
