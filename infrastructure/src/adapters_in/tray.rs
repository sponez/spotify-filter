use std::sync::mpsc::Sender;

use domain::ports::ports_in::events::AppRequest;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};

/// Wraps the system tray icon and its context menu.
/// Must be kept alive for the tray to remain visible.
pub struct TrayEventListener {
    _icon: TrayIcon,
    id_show: MenuId,
    id_sign_out: MenuId,
    id_quit: MenuId,
}

impl TrayEventListener {
    /// Create the tray icon from raw RGBA bytes.
    ///
    /// `icon_rgba` must be a flat `width × height × 4` byte buffer in RGBA order.
    pub fn new(icon_rgba: Vec<u8>, width: u32, height: u32) -> Self {
        let item_show = MenuItem::new("Show", true, None);
        let item_sign_out = MenuItem::new("Sign Out", true, None);
        let item_quit = MenuItem::new("Quit", true, None);

        let id_show = item_show.id().clone();
        let id_sign_out = item_sign_out.id().clone();
        let id_quit = item_quit.id().clone();

        let menu = Menu::new();
        menu.append(&item_show).unwrap();
        menu.append(&item_sign_out).unwrap();
        menu.append(&item_quit).unwrap();

        let icon = Icon::from_rgba(icon_rgba, width, height).expect("valid icon RGBA data");

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Spotify Filter")
            .with_icon(icon)
            .with_menu_on_left_click(false)
            .build()
            .expect("failed to build tray icon");

        Self {
            _icon: tray_icon,
            id_show,
            id_sign_out,
            id_quit,
        }
    }

    /// Drain all pending tray and menu events, forwarding them to the request channel.
    /// Call this once per timer tick from the GUI event loop.
    pub fn poll(&self, tx: &Sender<AppRequest>) {
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if matches!(event, TrayIconEvent::Click { .. }) {
                let _ = tx.send(AppRequest::ShowWindow);
            }
        }

        while let Ok(event) = MenuEvent::receiver().try_recv() {
            let id = &event.id;
            if *id == self.id_show {
                let _ = tx.send(AppRequest::ShowWindow);
            } else if *id == self.id_sign_out {
                let _ = tx.send(AppRequest::SignOut);
            } else if *id == self.id_quit {
                let _ = tx.send(AppRequest::Quit);
            }
        }
    }
}
