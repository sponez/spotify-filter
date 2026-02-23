use std::sync::mpsc::Sender;

use slint::ComponentHandle;
use tracing::info;

use domain::ports::ports_in::{
    events::AppRequest,
    settings::models::SaveSettingsCommand,
};

use crate::AppWindow;
use crate::window::mapper::settings_mapper::{slint_to_action_view, slint_to_target_view};

pub fn setup_open_settings_callback(window: &AppWindow, tx: Sender<AppRequest>) {
    window.on_open_settings(move || {
        info!("Open settings clicked");
        let _ = tx.send(AppRequest::GetSettings);
    });
}

pub fn setup_save_settings_callback(window: &AppWindow, tx: Sender<AppRequest>) {
    let w = window.as_weak();
    window.on_save_settings(move || {
        if let Some(w) = w.upgrade() {
            info!("Save settings clicked");
            let filter_action = slint_to_action_view(w.get_filter_action());
            let filter_target = slint_to_target_view(
                w.get_filter_target_type(),
                w.get_filter_playlist_index(),
            );
            let _ = tx.send(AppRequest::SaveSettings(SaveSettingsCommand {
                filter_action,
                filter_target,
            }));
        }
    });
}
