use global_hotkey::GlobalHotKeyManager;
use tray_icon::TrayIcon;

pub struct ApplicationContext {
    pub hotkeys_manager: Option<GlobalHotKeyManager>,
    pub tray_icon: Option<TrayIcon>,
}

impl ApplicationContext {
    pub fn new() -> Self {
        Self {
            hotkeys_manager: None,
            tray_icon: None,
        }
    }
}
