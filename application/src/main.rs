mod configuration;
mod context;
mod utils;

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
use global_hotkey::GlobalHotKeyManager;
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
use tray_icon::{Icon, TrayIconBuilder, menu::{Menu, MenuId, MenuItem}};

use crate::{context::ApplicationContext, utils::hotkey_parser::parse_hotkey};

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

fn setup_tray_icon(context: &mut ApplicationContext, icon_rgba: Vec<u8>, width: u32, height: u32) -> (MenuId, MenuId, MenuId) {
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

        context.tray_icon = Some(tray_icon);
        (id_show, id_sign_out, id_quit)
}

fn setup_hotkeys_manager(context: &mut ApplicationContext, filter_hotkey: &str, pass_hotkey: &str) -> (u32, u32) {
    let hotkey_filter = parse_hotkey(filter_hotkey);
    let hotkey_pass = parse_hotkey(pass_hotkey);

    let filter_id = hotkey_filter.id();
    let pass_id = hotkey_pass.id();

    let manager = GlobalHotKeyManager::new().expect("failed to create hotkey manager");
    manager
        .register(hotkey_filter)
        .expect("failed to register filter hotkey");
    manager
        .register(hotkey_pass)
        .expect("failed to register pass hotkey");

    context.hotkeys_manager = Some(manager);
    (filter_id, pass_id)
}

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let config = Configuration::load();
    let mut context = ApplicationContext::new();
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

    // Load tray icon
    let (icon_rgba, icon_width, icon_height) = load_icon_rgba();
    let (id_show, id_sign_out, id_quit) = setup_tray_icon(&mut context, icon_rgba, icon_width, icon_height);
    let tray_listener = Arc::new(TrayEventListener::new(id_show, id_sign_out, id_quit));
    tray_listener.start_polling(request_tx.clone());
    
    // Setup hotkeys
    let (id_filter, id_pass) = setup_hotkeys_manager(&mut context, &config.hotkeys.filter, &config.hotkeys.pass);
    let hotkey_listener = Arc::new(HotkeyEventListener::new(id_filter, id_pass));
    hotkey_listener.start_polling(request_tx.clone(), authorized);

    // Run GUI event loop
    gui::starter::run(request_tx, response_rx)
}
