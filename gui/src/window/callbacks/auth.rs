use std::sync::Arc;

use slint::{CloseRequestResponse, ComponentHandle};

use domain::ports::ports_in::spotify::usecases::{
    sign_in::SignInUseCase, sign_out::SignOutUseCase,
};

use crate::{AppStateEnum, AppWindow};

pub fn setup_close_handler(window: &AppWindow) {
    let window_weak = window.as_weak();
    window.window().on_close_requested(move || {
        if let Some(w) = window_weak.upgrade() {
            let state = w.get_state();
            if state == AppStateEnum::SignedIn || state == AppStateEnum::Settings {
                w.window().hide().ok();
                return CloseRequestResponse::KeepWindowShown;
            }
        }
        slint::quit_event_loop().ok();
        CloseRequestResponse::KeepWindowShown
    });
}

pub fn setup_sign_in_callback(window: &AppWindow, auth: Arc<dyn SignInUseCase>) {
    let window_weak = window.as_weak();
    window.on_sign_in(move || {
        if let Some(w) = window_weak.upgrade() {
            w.set_state(AppStateEnum::AwaitLogin);
            // TODO: sign_in blocks (waits for callback server) — move to a background
            // thread so the UI can show the spinner without freezing.
            if auth.sign_in().is_err() {
                w.set_state(AppStateEnum::Login);
                return;
            }
            w.set_state(AppStateEnum::SignedIn);
        }
    });
}

pub fn setup_sign_out_callback(window: &AppWindow, auth: Arc<dyn SignOutUseCase>) {
    let window_weak = window.as_weak();
    window.on_sign_out(move || {
        if let Some(w) = window_weak.upgrade() {
            if auth.sign_out().is_err() {
                return;
            }
            w.set_state(AppStateEnum::Login);
            w.window().show().ok();
        }
    });
}
