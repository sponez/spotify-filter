use std::sync::mpsc::Sender;

use slint::{CloseRequestResponse, ComponentHandle};

use domain::ports::ports_in::events::AppRequest;

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

pub fn setup_sign_in_callback(window: &AppWindow, tx: Sender<AppRequest>) {
    let window_weak = window.as_weak();
    window.on_sign_in(move || {
        if let Some(w) = window_weak.upgrade() {
            w.set_state(AppStateEnum::AwaitLogin);
            let _ = tx.send(AppRequest::SignIn);
        }
    });
}

pub fn setup_sign_out_callback(window: &AppWindow, tx: Sender<AppRequest>) {
    window.on_sign_out(move || {
        let _ = tx.send(AppRequest::SignOut);
    });
}
