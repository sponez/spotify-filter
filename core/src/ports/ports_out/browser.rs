use crate::errors::errors::AppResult;

pub trait BrowserLauncher: Send + Sync {
    fn open_url(&self, url: &str) -> AppResult<()>;
}
