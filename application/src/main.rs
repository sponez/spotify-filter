mod configuration;

use image::GenericImageView;

use configuration::configuration::Configuration;
use infrastructure::adapters_in::{hotkeys::HotkeyAdapter, tray::TrayAdapter};

fn load_icon_rgba() -> (Vec<u8>, u32, u32) {
    let bytes = include_bytes!("../resources/icon.png");
    let img = image::load_from_memory(bytes).expect("valid icon image");
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    (rgba, width, height)
}

fn main() {
    let config = Configuration::load();

    let hotkey_adapter = HotkeyAdapter::new(&config.hotkeys.discard, &config.hotkeys.like);

    let (icon_rgba, width, height) = load_icon_rgba();
    let tray_adapter = TrayAdapter::new(icon_rgba, width, height);

    // TODO: build service implementations and pass to gui::run::run(...)
    let _ = (tray_adapter, hotkey_adapter);
}
