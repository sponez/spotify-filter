pub trait ErrorNotification: Send + Sync {
    fn notify(&self, message: &str);
}
