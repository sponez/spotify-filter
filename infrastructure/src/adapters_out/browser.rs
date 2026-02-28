use domain::{errors::errors::AppResult, ports::ports_out::browser::BrowserLauncher};
use tracing::{error, info};

pub struct SystemBrowserLauncher;

impl BrowserLauncher for SystemBrowserLauncher {
    fn open_url(&self, url: &str) -> AppResult<()> {
        info!(url, "Opening URL in browser");
        open::that(url).map_err(|e| {
            error!(error = %e, "Failed to open browser");
            anyhow::anyhow!("Failed to open browser: {e}")
        })?;
        Ok(())
    }
}
