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
            auth.sign_in()
                .expect("Failed to sign in");
            // TODO: listen for auth completion and transition to SignedIn
            w.set_state(AppStateEnum::SignedIn);
        }
    });
}

pub fn setup_sign_out_callback(window: &AppWindow, auth: Arc<dyn SignOutUseCase>) {
    let window_weak = window.as_weak();
    window.on_sign_out(move || {
        if let Some(w) = window_weak.upgrade() {
            auth.sign_out()
                .expect("Failed to sign out");
            w.set_state(AppStateEnum::Login);
            w.window().show().ok();
        }
    });
}
