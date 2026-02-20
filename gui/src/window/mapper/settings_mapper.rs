use domain::ports::ports_in::settings::models::{FilterActionView, FilterTargetView, SettingsView};

use crate::{AppWindow, FilterActionEnum};

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

pub fn target_view_to_playlist_index(target: &FilterTargetView) -> i32 {
    match target {
        FilterTargetView::Playlist(_) => 0,
        FilterTargetView::LikedSongs => -1,
    }
}

pub fn slint_to_target_view(target_type: i32, _playlist_index: i32) -> FilterTargetView {
    match target_type {
        1 => FilterTargetView::Playlist(String::new()),
        _ => FilterTargetView::LikedSongs,
    }
}

pub fn apply_settings_view_to_window(w: &AppWindow, view: SettingsView) {
    w.set_filter_action(action_view_to_slint(&view.filter_action));
    w.set_filter_target_type(target_view_to_type(&view.filter_target));
    w.set_filter_playlist_index(target_view_to_playlist_index(&view.filter_target));
}
