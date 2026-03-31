use domain::ports::ports_in::settings::models::{PassActionView, PassTargetView};

use crate::adapters_out::repository::settings::dto::settings_dto::{
    FilterActionDto, PlaylistTargetDto, SettingsCacheDto, SettingsFileDto,
};

// ---- Cache ↔ View ----

pub fn cache_dto_to_view(dto: SettingsCacheDto) -> (PassActionView, PassTargetView) {
    (
        action_dto_to_view(dto.filter_action),
        target_dto_to_view(dto.filter_target),
    )
}

pub fn view_to_cache_dto(action: &PassActionView, target: &PassTargetView) -> SettingsCacheDto {
    SettingsCacheDto {
        filter_action: action_view_to_dto(action),
        filter_target: target_view_to_dto(action, target),
    }
}

// ---- File ↔ View ----

pub fn file_dto_to_view(dto: SettingsFileDto) -> (PassActionView, PassTargetView) {
    (
        action_dto_to_view(dto.filter_action),
        target_dto_to_view(dto.filter_target),
    )
}

pub fn view_to_file_dto(action: &PassActionView, target: &PassTargetView) -> SettingsFileDto {
    SettingsFileDto {
        filter_action: action_view_to_dto(action),
        filter_target: target_view_to_dto(action, target),
    }
}

// ---- Shared helpers ----

fn action_dto_to_view(dto: FilterActionDto) -> PassActionView {
    match dto {
        FilterActionDto::None => PassActionView::None,
        FilterActionDto::AddToPlaylist => PassActionView::AddToPlaylist,
        FilterActionDto::MoveToPlaylist => PassActionView::MoveToPlaylist,
    }
}

fn target_dto_to_view(dto: Option<PlaylistTargetDto>) -> PassTargetView {
    match dto {
        Some(PlaylistTargetDto::Playlist(id)) => PassTargetView::Playlist(id),
        _ => PassTargetView::LikedSongs,
    }
}

fn action_view_to_dto(action: &PassActionView) -> FilterActionDto {
    match action {
        PassActionView::None => FilterActionDto::None,
        PassActionView::AddToPlaylist => FilterActionDto::AddToPlaylist,
        PassActionView::MoveToPlaylist => FilterActionDto::MoveToPlaylist,
    }
}

fn target_view_to_dto(
    action: &PassActionView,
    target: &PassTargetView,
) -> Option<PlaylistTargetDto> {
    match action {
        PassActionView::None => None,
        _ => Some(match target {
            PassTargetView::Playlist(id) => PlaylistTargetDto::Playlist(id.clone()),
            PassTargetView::LikedSongs => PlaylistTargetDto::Liked,
        }),
    }
}
