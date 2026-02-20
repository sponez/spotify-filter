use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};

/// Wraps the system tray icon and its context menu.
/// Must be kept alive for the tray to remain visible.
pub struct TrayAdapter {
    _icon: TrayIcon,
    id_show: MenuId,
    id_sign_out: MenuId,
    id_quit: MenuId,
}

impl TrayAdapter {
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

    /// Drain all pending tray and menu events and call the appropriate callback.
    /// Call this once per timer tick from the GUI event loop.
    pub fn poll(
        &self,
        on_show: impl Fn(),
        on_sign_out: impl Fn(),
        on_quit: impl Fn(),
    ) {
        // Left-click on the tray icon → show window
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if matches!(event, TrayIconEvent::Click { .. }) {
                on_show();
            }
        }

        // Context menu items
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            let id = &event.id;
            if *id == self.id_show {
                on_show();
            } else if *id == self.id_sign_out {
                on_sign_out();
            } else if *id == self.id_quit {
                on_quit();
            }
        }
    }
}
