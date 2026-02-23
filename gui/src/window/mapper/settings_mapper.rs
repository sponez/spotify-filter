use std::cell::RefCell;

use slint::ModelRc;
use slint::VecModel;

use domain::ports::ports_in::settings::models::{FilterActionView, FilterTargetView, PlaylistItemView, SettingsView};

use crate::{AppWindow, FilterActionEnum};

thread_local! {
    static PLAYLISTS: RefCell<Vec<PlaylistItemView>> = RefCell::new(Vec::new());
}

pub fn action_view_to_slint(action: &FilterActionView) -> FilterActionEnum {
    match action {
        FilterActionView::None => FilterActionEnum::None,
        FilterActionView::AddToPlaylist => FilterActionEnum::AddToPlaylist,
        FilterActionView::MoveToPlaylist => FilterActionEnum::MoveToPlaylist,
    }
}

pub fn slint_to_action_view(e: FilterActionEnum) -> FilterActionView {
    match e {
        FilterActionEnum::None => FilterActionView::None,
        FilterActionEnum::AddToPlaylist => FilterActionView::AddToPlaylist,
        FilterActionEnum::MoveToPlaylist => FilterActionView::MoveToPlaylist,
    }
}

pub fn target_view_to_type(target: &FilterTargetView) -> i32 {
    match target {
        FilterTargetView::Playlist(_) => 1,
        FilterTargetView::LikedSongs => 0,
    }
}

pub fn slint_to_target_view(target_type: i32, playlist_index: i32) -> FilterTargetView {
    match target_type {
        1 => {
            let id = playlist_id_by_index(playlist_index).unwrap_or_default();
            FilterTargetView::Playlist(id)
        }
        _ => FilterTargetView::LikedSongs,
    }
}

fn playlist_id_by_index(index: i32) -> Option<String> {
    if index < 0 {
        return None;
    }
    PLAYLISTS.with(|p| {
        p.borrow().get(index as usize).map(|item| item.id.clone())
    })
}

pub fn apply_settings_view_to_window(w: &AppWindow, view: SettingsView) {
    w.set_filter_action(action_view_to_slint(&view.filter_action));
    w.set_filter_target_type(target_view_to_type(&view.filter_target));

    // Build playlist name model for the ComboBox
    let names: Vec<slint::SharedString> = view.playlists.iter()
        .map(|p| slint::SharedString::from(&p.name))
        .collect();
    w.set_filter_playlist_model(ModelRc::new(VecModel::from(names)));

    // Find selected index by matching the playlist ID
    let selected_index = match &view.filter_target {
        FilterTargetView::Playlist(id) if !id.is_empty() => {
            view.playlists.iter()
                .position(|p| p.id == *id)
                .map(|i| i as i32)
                .unwrap_or(-1)
        }
        _ => -1,
    };
    w.set_filter_playlist_index(selected_index);

    // Store playlists for later save lookup
    PLAYLISTS.with(|p| *p.borrow_mut() = view.playlists);
}
