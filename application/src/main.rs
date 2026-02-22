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
        ports_in::{
            events::{AppRequest, AppResponse},
            spotify::usecases::try_sign_in::TrySignInUseCase,
        },
        ports_out::{
            auth::{auth_url_builder::AuthUrlBuilder, pkce::PkceGenerator},
            browser::BrowserLauncher,
            client::spotify_auth::SpotifyAuthClient,
            notification::ErrorNotification,
            repository::{
                settings::{SettingsCache, SettingsStore},
                token::{RefreshTokenStore, TokenCache},
            },
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
            try_sign_in::TrySignInInteractor,
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
        auth::{
            pkce::Sha256PkceGenerator,
            spotify_auth_url::SpotifyAuthUrlBuilder,
        },
        browser::SystemBrowserLauncher,
        client::spotify::spotify_auth_client::UreqSpotifyAuthClient,
        notification::ToastErrorNotification,
        repository::{
            settings::{
                cache::LocalSettingsCache,
                file::JsonFileSettingsStore,
            },
            token::{
                cache::LocalTokenCache,
                keyring::KeyringRefreshTokenStore,
            },
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

struct AuthUseCases {
    sign_in: Arc<SignInInteractor>,
    sign_out: Arc<SignOutInteractor>,
    try_sign_in: Arc<TrySignInInteractor>,
}

fn build_auth_use_cases(
    config: &Configuration,
    notifier: Arc<dyn ErrorNotification>,
) -> AuthUseCases {
    let parsed = url::Url::parse(&config.app.spotify.auth.redirect_uri)
        .unwrap_or_else(|e| panic!("invalid redirect_uri '{}': {e}", config.app.spotify.auth.redirect_uri));
    let addr = format!(
        "{}:{}",
        parsed.host_str().expect("redirect_uri must have a host"),
        parsed.port().expect("redirect_uri must have a port"),
    );
    let path = parsed.path().to_string();

    let callback_server: Box<dyn CallbackServer> = Box::new(TinyHttpCallbackServer::new(addr, path));
    let pkce_generator: Arc<dyn PkceGenerator> = Arc::new(Sha256PkceGenerator);
    let auth_url_builder: Arc<dyn AuthUrlBuilder> = Arc::new(SpotifyAuthUrlBuilder::new(
        config.app.spotify.auth.auth_uri.clone(),
        config.app.spotify.auth.client_id.clone(),
        config.app.spotify.auth.redirect_uri.clone(),
        config.app.spotify.auth.scopes.clone(),
    ));
    let browser: Arc<dyn BrowserLauncher> = Arc::new(SystemBrowserLauncher);
    let auth_client: Arc<dyn SpotifyAuthClient> = Arc::new(UreqSpotifyAuthClient::new(
        config.app.spotify.auth.token_uri.clone(),
        config.app.spotify.auth.client_id.clone(),
        config.app.spotify.auth.redirect_uri.clone(),
    ));
    let token_cache: Arc<dyn TokenCache> = Arc::new(LocalTokenCache::new());
    let refresh_token_store: Arc<dyn RefreshTokenStore> = Arc::new(
        KeyringRefreshTokenStore::new("spotify-filter".into(), "refresh_token".into()),
    );

    let sign_in = Arc::new(SignInInteractor::new(
        callback_server,
        pkce_generator,
        auth_url_builder,
        browser,
        Arc::clone(&auth_client),
        Arc::clone(&token_cache),
        Arc::clone(&refresh_token_store),
        Arc::clone(&notifier),
    ));

    let sign_out = Arc::new(SignOutInteractor::new(
        Arc::clone(&token_cache),
        Arc::clone(&refresh_token_store),
        notifier,
    ));

    let try_sign_in = Arc::new(TrySignInInteractor::new(
        auth_client,
        token_cache,
        refresh_token_store,
    ));

    AuthUseCases { sign_in, sign_out, try_sign_in }
}

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let config = Configuration::load();
    let mut context = ApplicationContext::new();
    let notifier: Arc<dyn ErrorNotification> = Arc::new(ToastErrorNotification::new());

    // Auth use-cases
    let auth = build_auth_use_cases(&config, Arc::clone(&notifier));
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

    // Try silent sign-in before starting the UI
    let initially_authorized = matches!(auth.try_sign_in.try_sign_in(), Ok(true));
    if initially_authorized {
        authorized.store(true, std::sync::atomic::Ordering::Relaxed);
        notifier.notify("Spotify Filter is ready");
    }

    // Event dispatcher thread
    let dispatcher = EventDispatcher::new(
        request_rx,
        response_tx,
        Arc::clone(&authorized),
        auth.sign_in,
        auth.sign_out,
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
    gui::starter::run(request_tx, response_rx, initially_authorized)
}
