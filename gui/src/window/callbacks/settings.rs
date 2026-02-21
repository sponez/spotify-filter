use std::sync::Arc;

use domain::ports::ports_in::settings::{
    models::SaveSettingsCommand,
    usecases::{get_settings::GetSettingsUseCase, save_settings::SaveSettingsUseCase},
};

use slint::ComponentHandle;

use crate::{AppStateEnum, AppWindow};
use crate::window::mapper::settings_mapper::{
    apply_settings_view_to_window, slint_to_action_view, slint_to_target_view,
};

pub fn setup_open_settings_callback(window: &AppWindow, get_settings: Arc<dyn GetSettingsUseCase>) {
    let w = window.as_weak();
    window.on_open_settings(move || {
        if let Some(w) = w.upgrade() {
            match get_settings.get_settings() {
                Ok(view) => {
                    apply_settings_view_to_window(&w, view);
                    w.set_state(AppStateEnum::Settings);
                }
                Err(_) => {}
            }
        }
    });
}

pub fn setup_save_settings_callback(window: &AppWindow, save_settings: Arc<dyn SaveSettingsUseCase>) {
    let w = window.as_weak();
    window.on_save_settings(move || {
        if let Some(w) = w.upgrade() {
            let filter_action = slint_to_action_view(w.get_filter_action());
            let filter_target = slint_to_target_view(
                w.get_filter_target_type(),
                w.get_filter_playlist_index(),
            );
            if save_settings.save_settings(SaveSettingsCommand { filter_action, filter_target }).is_ok() {
                w.set_state(AppStateEnum::SignedIn);
            }
        }
    });
}
