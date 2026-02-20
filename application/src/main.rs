mod configuration;
mod settings;

use std::sync::Arc;

use domain::{
    ports::ports_in::spotify::spotify_facade::SpotifyFacade,
    usecases::spotify::{
        filter_track::FilterTrackInteractor,
        pass_track::PassTrackInteractor,
        sign_in::SignInInteractor,
        sign_out::SignOutInteractor
    }
};
use gui::run;
use image::GenericImageView;

use configuration::configuration::Configuration;
use settings::settings::Settings;
use infrastructure::adapters_in::{hotkeys::HotkeyAdapter, tray::TrayAdapter};

fn load_icon_rgba() -> (Vec<u8>, u32, u32) {
    let bytes = include_bytes!("../resources/icon.png");
    let img = image::load_from_memory(bytes).expect("valid icon image");
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    (rgba, width, height)
}

fn create_spotify_facade() -> SpotifyFacade {
    let sign_in = SignInInteractor::new();
    let sign_out = SignOutInteractor::new();
    let pass_track = PassTrackInteractor::new();
    let filter_track = FilterTrackInteractor::new();

    SpotifyFacade::new(
        Arc::new(sign_in),
        Arc::new(sign_out),
        Arc::new(pass_track),
        Arc::new(filter_track)
    )
}

fn main() -> Result<(), slint::PlatformError> {
    let config = Configuration::load();
    let _settings = Settings::load();

    let hotkey_adapter = HotkeyAdapter::new(&config.hotkeys.filter, &config.hotkeys.pass);

    let (icon_rgba, width, height) = load_icon_rgba();
    let tray_adapter = TrayAdapter::new(icon_rgba, width, height);
    let spotify_facade = create_spotify_facade();

    run::run(tray_adapter, hotkey_adapter, spotify_facade)
}
