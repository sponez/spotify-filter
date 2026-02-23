use domain::ports::ports_out::notification::ErrorNotification;
use tracing::error;
use winrt_notification::Toast;

pub struct ToastErrorNotification;

impl ToastErrorNotification {
    pub fn new() -> Self {
        Self
    }
}

impl ErrorNotification for ToastErrorNotification {
    fn notify(&self, message: &str) {
        error!(message, "Showing error notification");
        Toast::new(Toast::POWERSHELL_APP_ID)
            .title("Spotify Filter")
            .text1(message)
            .show()
            .ok();
    }
}
