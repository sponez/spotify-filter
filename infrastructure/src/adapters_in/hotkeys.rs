use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
};

use domain::ports::ports_in::events::AppRequest;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

/// Parse a hotkey string like `"Ctrl+Alt+D"` into a [`HotKey`].
///
/// Supported modifier names: `Ctrl`, `Alt`, `Shift`, `Super`.
/// The final token is treated as the key code (e.g. `D` → `Code::KeyD`).
pub fn parse_hotkey(s: &str) -> HotKey {
    let mut mods = Modifiers::empty();
    let mut key: Option<Code> = None;

    for token in s.split('+') {
        match token.trim() {
            "Ctrl" | "Control" => mods |= Modifiers::CONTROL,
            "Alt" => mods |= Modifiers::ALT,
            "Shift" => mods |= Modifiers::SHIFT,
            "Super" | "Meta" => mods |= Modifiers::SUPER,
            k => {
                key = Some(parse_code(k));
            }
        }
    }

    let code = key.unwrap_or_else(|| panic!("hotkey string '{s}' has no key code"));
    let mods_opt = if mods.is_empty() { None } else { Some(mods) };
    HotKey::new(mods_opt, code)
}

fn parse_code(s: &str) -> Code {
    match s {
        "A" => Code::KeyA,
        "B" => Code::KeyB,
        "C" => Code::KeyC,
        "D" => Code::KeyD,
        "E" => Code::KeyE,
        "F" => Code::KeyF,
        "G" => Code::KeyG,
        "H" => Code::KeyH,
        "I" => Code::KeyI,
        "J" => Code::KeyJ,
        "K" => Code::KeyK,
        "L" => Code::KeyL,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "O" => Code::KeyO,
        "P" => Code::KeyP,
        "Q" => Code::KeyQ,
        "R" => Code::KeyR,
        "S" => Code::KeyS,
        "T" => Code::KeyT,
        "U" => Code::KeyU,
        "V" => Code::KeyV,
        "W" => Code::KeyW,
        "X" => Code::KeyX,
        "Y" => Code::KeyY,
        "Z" => Code::KeyZ,
        "0" => Code::Digit0,
        "1" => Code::Digit1,
        "2" => Code::Digit2,
        "3" => Code::Digit3,
        "4" => Code::Digit4,
        "5" => Code::Digit5,
        "6" => Code::Digit6,
        "7" => Code::Digit7,
        "8" => Code::Digit8,
        "9" => Code::Digit9,
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        "Space" => Code::Space,
        "Enter" => Code::Enter,
        "Escape" => Code::Escape,
        "Tab" => Code::Tab,
        "Backspace" => Code::Backspace,
        "Delete" => Code::Delete,
        "Home" => Code::Home,
        "End" => Code::End,
        "PageUp" => Code::PageUp,
        "PageDown" => Code::PageDown,
        "ArrowUp" => Code::ArrowUp,
        "ArrowDown" => Code::ArrowDown,
        "ArrowLeft" => Code::ArrowLeft,
        "ArrowRight" => Code::ArrowRight,
        other => panic!("unknown key code '{other}' in hotkey string"),
    }
}

/// Owns the [`GlobalHotKeyManager`] and registered hotkeys.
/// Must be kept alive for hotkeys to remain active.
pub struct HotkeyEventListener {
    _manager: GlobalHotKeyManager,
    filter_id: u32,
    pass_id: u32,
}

impl HotkeyEventListener {
    /// Register hotkeys from strings like `"Ctrl+Alt+D"`.
    pub fn new(filter: &str, pass: &str) -> Self {
        let hotkey_filter = parse_hotkey(filter);
        let hotkey_pass = parse_hotkey(pass);

        let filter_id = hotkey_filter.id();
        let pass_id = hotkey_pass.id();

        let manager = GlobalHotKeyManager::new().expect("failed to create hotkey manager");
        manager
            .register(hotkey_filter)
            .expect("failed to register filter hotkey");
        manager
            .register(hotkey_pass)
            .expect("failed to register pass hotkey");

        Self {
            _manager: manager,
            filter_id,
            pass_id,
        }
    }

    /// Drain all pending hotkey events, forwarding them to the request channel.
    /// Only sends events when `authorized` is true.
    /// Call this once per timer tick from the GUI event loop.
    pub fn poll(&self, tx: &Sender<AppRequest>, authorized: &Arc<AtomicBool>) {
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state != HotKeyState::Pressed {
                continue;
            }
            if !authorized.load(Ordering::Relaxed) {
                continue;
            }
            if event.id == self.filter_id {
                let _ = tx.send(AppRequest::FilterTrack);
            } else if event.id == self.pass_id {
                let _ = tx.send(AppRequest::PassTrack);
            }
        }
    }
}
