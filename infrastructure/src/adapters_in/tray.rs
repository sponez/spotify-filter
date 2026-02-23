use std::sync::{Arc, mpsc::Sender};
use tracing::{debug, info};

use domain::ports::ports_in::events::AppRequest;
use tray_icon::{
    TrayIconEvent,
    menu::{MenuEvent, MenuId},
};

/// Wraps the system tray icon and its context menu.
/// Must be kept alive for the tray to remain visible.
pub struct TrayEventListener {
    id_show: MenuId,
    id_sign_out: MenuId,
    id_quit: MenuId,
}

impl TrayEventListener {
    /// Create the tray icon from raw RGBA bytes.
    ///
    /// `icon_rgba` must be a flat `width × height × 4` byte buffer in RGBA order.
    pub fn new(id_show: MenuId, id_sign_out: MenuId, id_quit: MenuId) -> Self {
        Self {
            id_show,
            id_sign_out,
            id_quit,
        }
    }

    pub fn start_polling(self: Arc<Self>, tx: Sender<AppRequest>) {
        info!("Starting tray polling thread");
        std::thread::spawn(move || loop {
            self.poll(&tx);
            std::thread::sleep(std::time::Duration::from_millis(50));
        });
    }

    /// Drain all pending tray and menu events, forwarding them to the request channel.
    /// Call this once per timer tick from the GUI event loop.
    pub fn poll(&self, tx: &Sender<AppRequest>) {
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if matches!(event, TrayIconEvent::Click { .. }) {
                debug!("Tray icon clicked");
                let _ = tx.send(AppRequest::ShowWindow);
            }
        }

        while let Ok(event) = MenuEvent::receiver().try_recv() {
            let id = &event.id;
            if *id == self.id_show {
                debug!("Tray menu: show");
                let _ = tx.send(AppRequest::ShowWindow);
            } else if *id == self.id_sign_out {
                debug!("Tray menu: sign out");
                let _ = tx.send(AppRequest::SignOut);
            } else if *id == self.id_quit {
                info!("Tray menu: quit");
                let _ = tx.send(AppRequest::Quit);
            }
        }
    }
}
