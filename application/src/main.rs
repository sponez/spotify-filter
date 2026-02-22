mod configuration;

use std::sync::{
    Arc,
    atomic::AtomicBool,
    mpsc,
};

use domain::{
    ports::{
        ports_in::events::{AppRequest, AppResponse},
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
    adapters_in::{
        event_dispatcher::EventDispatcher,
        hotkeys::HotkeyEventListener,
        tray::TrayEventListener,
    },
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

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let config = Configuration::load();
    let notifier: Arc<dyn ErrorNotification> = Arc::new(ToastErrorNotification::new());

    let (addr, path) = parse_redirect_uri(&config.app.spotify.auth.redirect_uri);
    let callback_server: Box<dyn CallbackServer> = Box::new(TinyHttpCallbackServer::new(addr, path));

    // Use-cases
    let sign_in = Arc::new(SignInInteractor::new(callback_server, Arc::clone(&notifier)));
    let sign_out = Arc::new(SignOutInteractor::new(Arc::clone(&notifier)));
    let pass_track = Arc::new(PassTrackInteractor::new(Arc::clone(&notifier)));
    let filter_track = Arc::new(FilterTrackInteractor::new(Arc::clone(&notifier)));

    let cache: Arc<dyn SettingsCache> = Arc::new(LocalSettingsCache::new());
    let file: Arc<dyn SettingsStore> = Arc::new(JsonFileSettingsStore::new());
    let get_settings = Arc::new(GetSettingsInteractor::new(Arc::clone(&cache), Arc::clone(&file), Arc::clone(&notifier)));
    let save_settings = Arc::new(SaveSettingsInteractor::new(cache, file, Arc::clone(&notifier)));

    // Channels
    let (request_tx, request_rx) = mpsc::channel::<AppRequest>();
    let (response_tx, response_rx) = mpsc::channel::<AppResponse>();

    // Shared auth state
    let authorized = Arc::new(AtomicBool::new(false));

    // Event dispatcher thread
    let dispatcher = EventDispatcher::new(
        request_rx,
        response_tx,
        Arc::clone(&authorized),
        sign_in,
        sign_out,
        filter_track,
        pass_track,
        get_settings,
        save_settings,
    );
    std::thread::spawn(move || dispatcher.run());

    // Tray and hotkeys (must live on GUI thread for Windows message pump)
    let hotkey_listener = HotkeyEventListener::new(&config.hotkeys.filter, &config.hotkeys.pass);
    let (icon_rgba, width, height) = load_icon_rgba();
    let tray_listener = TrayEventListener::new(icon_rgba, width, height);

    // Run GUI event loop
    gui::starter::run(tray_listener, hotkey_listener, authorized, request_tx, response_rx)
}
