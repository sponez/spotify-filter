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
            server::callback_server::CallbackServer,
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
        server::callback_server::TinyHttpCallbackServer,
    },
};

fn load_icon_rgba() -> (Vec<u8>, u32, u32) {
    let bytes = include_bytes!("../resources/icon.png");
    let img = image::load_from_memory(bytes).expect("valid icon image");
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    (rgba, width, height)
}

fn parse_redirect_uri(uri: &str) -> (String, String) {
    let parsed = url::Url::parse(uri)
        .unwrap_or_else(|e| panic!("invalid redirect_uri '{uri}': {e}"));
    let addr = format!(
        "{}:{}",
        parsed.host_str().expect("redirect_uri must have a host"),
        parsed.port().expect("redirect_uri must have a port"),
    );
    let path = parsed.path().to_string();
    (addr, path)
}

fn create_spotify_facade(
    callback_server: Box<dyn CallbackServer>,
    notifier: &Arc<dyn ErrorNotification>,
) -> SpotifyFacade {
    SpotifyFacade::new(
        Arc::new(SignInInteractor::new(callback_server, Arc::clone(notifier))),
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

    let (addr, path) = parse_redirect_uri(&config.app.spotify.auth.redirect_uri);
    let callback_server: Box<dyn CallbackServer> = Box::new(TinyHttpCallbackServer::new(addr, path));

    let hotkey_listener = HotkeyEventListener::new(&config.hotkeys.filter, &config.hotkeys.pass);

    let (icon_rgba, width, height) = load_icon_rgba();
    let tray_listener = TrayEventListener::new(icon_rgba, width, height);

    gui::starter::run(
        tray_listener,
        hotkey_listener,
        create_spotify_facade(callback_server, &notifier),
        create_settings_facade(&notifier),
    )
}
