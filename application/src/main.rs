mod configuration;

use std::sync::Arc;

use domain::{
    ports::{
        ports_in::{
            settings::settings_facade::SettingsFacade,
            spotify::spotify_facade::SpotifyFacade,
        },
        ports_out::{
            notification::ErrorNotification,
            repository::settings::{SettingsCache, SettingsStore},
        },
    },
    usecases::{
        settings::{
            get_settings::GetSettingsInteractor,
            save_settings::SaveSettingsInteractor,
        },
        spotify::{
            filter_track::FilterTrackInteractor,
            pass_track::PassTrackInteractor,
            sign_in::SignInInteractor,
            sign_out::SignOutInteractor,
        },
    },
};
use gui;
use image::GenericImageView;

use configuration::configuration::Configuration;
use infrastructure::{
    adapters_in::{hotkeys::HotkeyEventListener, tray::TrayEventListener},
    adapters_out::{
        notification::ToastErrorNotification,
        repository::settings::{
            cache::LocalSettingsCache,
            file::JsonFileSettingsStore,
        },
    },
};

fn load_icon_rgba() -> (Vec<u8>, u32, u32) {
    let bytes = include_bytes!("../resources/icon.png");
    let img = image::load_from_memory(bytes).expect("valid icon image");
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    (rgba, width, height)
}

fn create_spotify_facade(notifier: &Arc<dyn ErrorNotification>) -> SpotifyFacade {
    SpotifyFacade::new(
        Arc::new(SignInInteractor::new(Arc::clone(notifier))),
        Arc::new(SignOutInteractor::new(Arc::clone(notifier))),
        Arc::new(PassTrackInteractor::new(Arc::clone(notifier))),
        Arc::new(FilterTrackInteractor::new(Arc::clone(notifier))),
    )
}

fn create_settings_facade(notifier: &Arc<dyn ErrorNotification>) -> SettingsFacade {
    let cache: Arc<dyn SettingsCache> = Arc::new(LocalSettingsCache::new());
    let file: Arc<dyn SettingsStore> = Arc::new(JsonFileSettingsStore::new());
    SettingsFacade::new(
        Arc::new(GetSettingsInteractor::new(Arc::clone(&cache), Arc::clone(&file), Arc::clone(notifier))),
        Arc::new(SaveSettingsInteractor::new(cache, file, Arc::clone(notifier))),
    )
}

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let config = Configuration::load();
    let notifier: Arc<dyn ErrorNotification> = Arc::new(ToastErrorNotification::new());

    let hotkey_listener = HotkeyEventListener::new(&config.hotkeys.filter, &config.hotkeys.pass);

    let (icon_rgba, width, height) = load_icon_rgba();
    let tray_listener = TrayEventListener::new(icon_rgba, width, height);

    gui::starter::run(
        tray_listener,
        hotkey_listener,
        create_spotify_facade(&notifier),
        create_settings_facade(&notifier),
    )
}
