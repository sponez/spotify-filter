use domain::ports::ports_in::settings::models::{FilterActionView, FilterTargetView};

use crate::adapters_out::repository::settings::dto::settings_dto::{
    FilterActionDto, PlaylistTargetDto, SettingsCacheDto, SettingsFileDto,
};

// ---- Cache ↔ View ----

pub fn cache_dto_to_view(dto: SettingsCacheDto) -> (FilterActionView, FilterTargetView) {
    (action_dto_to_view(dto.filter_action), target_dto_to_view(dto.filter_target))
}

pub fn view_to_cache_dto(action: &FilterActionView, target: &FilterTargetView) -> SettingsCacheDto {
    SettingsCacheDto {
        filter_action: action_view_to_dto(action),
        filter_target: target_view_to_dto(action, target),
    }
}

// ---- File ↔ View ----

pub fn file_dto_to_view(dto: SettingsFileDto) -> (FilterActionView, FilterTargetView) {
    (action_dto_to_view(dto.filter_action), target_dto_to_view(dto.filter_target))
}

pub fn view_to_file_dto(action: &FilterActionView, target: &FilterTargetView) -> SettingsFileDto {
    SettingsFileDto {
        filter_action: action_view_to_dto(action),
        filter_target: target_view_to_dto(action, target),
    }
}

// ---- Shared helpers ----

fn action_dto_to_view(dto: FilterActionDto) -> FilterActionView {
    match dto {
        FilterActionDto::None => FilterActionView::None,
        FilterActionDto::AddToPlaylist => FilterActionView::AddToPlaylist,
        FilterActionDto::MoveToPlaylist => FilterActionView::MoveToPlaylist,
    }
}

fn target_dto_to_view(dto: Option<PlaylistTargetDto>) -> FilterTargetView {
    match dto {
        Some(PlaylistTargetDto::Playlist(id)) => FilterTargetView::Playlist(id),
        _ => FilterTargetView::LikedSongs,
    }
}

fn action_view_to_dto(action: &FilterActionView) -> FilterActionDto {
    match action {
        FilterActionView::None => FilterActionDto::None,
        FilterActionView::AddToPlaylist => FilterActionDto::AddToPlaylist,
        FilterActionView::MoveToPlaylist => FilterActionDto::MoveToPlaylist,
    }
}

fn target_view_to_dto(action: &FilterActionView, target: &FilterTargetView) -> Option<PlaylistTargetDto> {
    match action {
        FilterActionView::None => None,
        _ => Some(match target {
            FilterTargetView::Playlist(id) => PlaylistTargetDto::Playlist(id.clone()),
            FilterTargetView::LikedSongs => PlaylistTargetDto::Liked,
        }),
    }
}
