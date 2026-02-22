use domain::{
    errors::errors::AppResult,
    ports::ports_out::browser::BrowserLauncher,
};

pub struct SystemBrowserLauncher;

impl BrowserLauncher for SystemBrowserLauncher {
    fn open_url(&self, url: &str) -> AppResult<()> {
        open::that(url)
            .map_err(|e| anyhow::anyhow!("Failed to open browser: {e}"))?;
        Ok(())
    }
}
