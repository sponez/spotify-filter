use std::cell::RefCell;

use slint::ModelRc;
use slint::VecModel;

use domain::ports::ports_in::settings::models::{PassActionView, PassTargetView, PlaylistItemView, SettingsView};

use crate::{AppWindow, FilterActionEnum};

thread_local! {
    static PLAYLISTS: RefCell<Vec<PlaylistItemView>> = RefCell::new(Vec::new());
}

pub fn action_view_to_slint(action: &PassActionView) -> FilterActionEnum {
    match action {
        PassActionView::None => FilterActionEnum::None,
        PassActionView::AddToPlaylist => FilterActionEnum::AddToPlaylist,
        PassActionView::MoveToPlaylist => FilterActionEnum::MoveToPlaylist,
    }
}

pub fn slint_to_action_view(e: FilterActionEnum) -> PassActionView {
    match e {
        FilterActionEnum::None => PassActionView::None,
        FilterActionEnum::AddToPlaylist => PassActionView::AddToPlaylist,
        FilterActionEnum::MoveToPlaylist => PassActionView::MoveToPlaylist,
    }
}

pub fn target_view_to_type(target: &PassTargetView) -> i32 {
    match target {
        PassTargetView::Playlist(_) => 1,
        PassTargetView::LikedSongs => 0,
    }
}

pub fn slint_to_target_view(target_type: i32, playlist_index: i32) -> PassTargetView {
    match target_type {
        1 => {
            let id = playlist_id_by_index(playlist_index).unwrap_or_default();
            PassTargetView::Playlist(id)
        }
        _ => PassTargetView::LikedSongs,
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
    w.set_filter_action(action_view_to_slint(&view.pass_action));
    w.set_filter_target_type(target_view_to_type(&view.pass_target));

    // Store the target for playlist index resolution after playlists arrive
    SELECTED_TARGET.with(|t| *t.borrow_mut() = Some(view.pass_target));
}

thread_local! {
    static SELECTED_TARGET: RefCell<Option<PassTargetView>> = RefCell::new(None);
}

pub fn apply_playlists_to_window(w: &AppWindow, mut playlists: Vec<PlaylistItemView>) {
    // Resolve selected index, inserting "playlist deleted" placeholder if needed
    let selected_index = SELECTED_TARGET.with(|t| {
        match t.borrow().as_ref() {
            Some(PassTargetView::Playlist(id)) if !id.is_empty() => {
                match playlists.iter().position(|p| p.id == *id) {
                    Some(i) => i as i32,
                    None => {
                        playlists.insert(0, PlaylistItemView {
                            id: id.clone(),
                            name: "playlist deleted".to_string(),
                        });
                        0
                    }
                }
            }
            _ => -1,
        }
    });

    // Set index BEFORE model to prevent Slint's auto-reset to 0
    w.set_filter_playlist_index(selected_index);

    let names: Vec<slint::SharedString> = playlists.iter()
        .map(|p| slint::SharedString::from(&p.name))
        .collect();
    w.set_filter_playlist_model(ModelRc::new(VecModel::from(names)));

    // Store playlists for later save lookup
    PLAYLISTS.with(|p| *p.borrow_mut() = playlists);
}
