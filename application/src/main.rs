mod configuration;
mod settings;

use std::sync::{Arc, Mutex};

use domain::{
    ports::ports_in::spotify::spotify_facade::SpotifyFacade,
    usecases::spotify::{
        filter_track::FilterTrackInteractor,
        pass_track::PassTrackInteractor,
        sign_in::SignInInteractor,
        sign_out::SignOutInteractor,
    },
};
use gui::run;
use image::GenericImageView;

use configuration::configuration::Configuration;
use settings::models::filter_action::{AdditionalFilterAction, PlaylistTarget};
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
        Arc::new(filter_track),
    )
}

fn action_to_int(a: &AdditionalFilterAction) -> i32 {
    match a {
        AdditionalFilterAction::None => 0,
        AdditionalFilterAction::AddToPlaylist => 1,
        AdditionalFilterAction::MoveToPlaylist => 2,
    }
}

fn int_to_action(n: i32) -> AdditionalFilterAction {
    match n {
        1 => AdditionalFilterAction::AddToPlaylist,
        2 => AdditionalFilterAction::MoveToPlaylist,
        _ => AdditionalFilterAction::None,
    }
}

/// Returns (target_type, playlist_index).
/// target_type: 0=Liked, 1=Playlist. playlist_index is -1 when not applicable.
fn target_to_ints(t: &Option<PlaylistTarget>) -> (i32, i32) {
    match t {
        None | Some(PlaylistTarget::Liked) => (0, -1),
        Some(PlaylistTarget::Playlist(_)) => (1, 0),
    }
}

fn ints_to_target(
    action: &AdditionalFilterAction,
    tt: i32,
    _playlist_index: i32,
) -> Option<PlaylistTarget> {
    match action {
        AdditionalFilterAction::None => None,
        _ => match tt {
            1 => Some(PlaylistTarget::Playlist(String::new())), // ID filled when Spotify fetch lands
            _ => Some(PlaylistTarget::Liked),
        },
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let config = Configuration::load();
    let settings = Settings::load();

    let initial_action = action_to_int(&settings.filter_action);
    let (initial_target_type, initial_playlist_index) = target_to_ints(&settings.filter_target);

    let current_settings = Arc::new(Mutex::new((
        initial_action,
        initial_target_type,
        initial_playlist_index,
    )));
    let shared_settings = Arc::new(Mutex::new(settings));

    let on_save: Box<dyn Fn(i32, i32, i32) + 'static> = {
        let shared = Arc::clone(&shared_settings);
        let current = Arc::clone(&current_settings);
        Box::new(move |a, tt, pi| {
            let action = int_to_action(a);
            let target = ints_to_target(&action, tt, pi);
            let new_s = Settings { filter_action: action, filter_target: target };
            new_s.save();
            *shared.lock().unwrap() = new_s;
            *current.lock().unwrap() = (a, tt, pi);
        })
    };

    let hotkey_adapter = HotkeyAdapter::new(&config.hotkeys.filter, &config.hotkeys.pass);

    let (icon_rgba, width, height) = load_icon_rgba();
    let tray_adapter = TrayAdapter::new(icon_rgba, width, height);
    let spotify_facade = create_spotify_facade();

    run::run(tray_adapter, hotkey_adapter, spotify_facade, current_settings, on_save)
}
