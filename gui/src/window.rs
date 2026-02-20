use std::sync::Arc;

use slint::{CloseRequestResponse, ComponentHandle};

use crate::{AppStateEnum, AppWindow};

use domain::ports::ports_in::spotify;

pub fn setup_close_handler(window: &AppWindow) {
    let window_weak = window.as_weak();
    window.window().on_close_requested(move || {
        if let Some(w) = window_weak.upgrade() {
            if w.get_state() == AppStateEnum::SignedIn {
                w.window().hide().ok();
                return CloseRequestResponse::KeepWindowShown;
            }
        }
        slint::quit_event_loop().ok();
        CloseRequestResponse::KeepWindowShown
    });
}

pub fn setup_sign_in_callback(window: &AppWindow, auth: Arc<dyn spotify::SignInUseCase>) {
    let window_weak = window.as_weak();
    window.on_sign_in(move || {
        if let Some(w) = window_weak.upgrade() {
            w.set_state(AppStateEnum::AwaitLogin);
            auth.sign_in();
            // TODO: listen for auth completion and transition to SignedIn
        }
    });
}

pub fn setup_sign_out_callback(window: &AppWindow, auth: Arc<dyn spotify::SignOutUseCase>) {
    let window_weak = window.as_weak();
    window.on_sign_out(move || {
        if let Some(w) = window_weak.upgrade() {
            auth.sign_out();
            w.set_state(AppStateEnum::Login);
            w.window().show().ok();
        }
    });
}
