use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;
use std::time::Duration;

use domain::{
    errors::errors::AppResult,
    ports::ports_out::{notification::ErrorNotification, repository::token::TokenCache},
};
use indexmap::IndexSet;
use tracing::{error, info};

use crate::adapters_out::client::spotify::{
    action::SpotifyApiAction,
    request_scheduler::{RequestScheduler, ScheduleMode},
};

const CRON_INTERVAL: Duration = Duration::from_secs(3600);
const PHASE_GAP: Duration = Duration::from_secs(35);
pub(crate) const PLAYLIST_API_INTERVAL: Duration = Duration::from_secs(35);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum QueueTarget {
    Liked,
    Playlist(String),
}

pub(crate) type QueueMap = HashMap<QueueTarget, IndexSet<String>>;

pub(crate) struct PlaylistSyncScheduler {
    stop_tx: Mutex<Option<mpsc::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

struct CycleContext<'a> {
    base_url: &'a str,
    paths: &'a HashMap<SpotifyApiAction, String>,
    token_cache: &'a Arc<dyn TokenCache>,
    notifier: &'a Arc<dyn ErrorNotification>,
    scheduler: &'a Arc<RequestScheduler>,
    add_queue: &'a Arc<Mutex<QueueMap>>,
    remove_queue: &'a Arc<Mutex<QueueMap>>,
}

impl PlaylistSyncScheduler {
    pub(crate) fn start(
        base_url: String,
        paths: HashMap<SpotifyApiAction, String>,
        token_cache: Arc<dyn TokenCache>,
        notifier: Arc<dyn ErrorNotification>,
        scheduler: Arc<RequestScheduler>,
        add_queue: Arc<Mutex<QueueMap>>,
        remove_queue: Arc<Mutex<QueueMap>>,
    ) -> Self {
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let handle = std::thread::spawn(move || {
            loop {
                match stop_rx.recv_timeout(CRON_INTERVAL) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        info!("Stopping playlist sync scheduler, flushing queues before exit");
                        let ctx = CycleContext {
                            base_url: &base_url,
                            paths: &paths,
                            token_cache: &token_cache,
                            notifier: &notifier,
                            scheduler: &scheduler,
                            add_queue: &add_queue,
                            remove_queue: &remove_queue,
                        };
                        Self::run_cycle(&ctx, None);
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        notifier.notify("Playlist sync started");
                        let ctx = CycleContext {
                            base_url: &base_url,
                            paths: &paths,
                            token_cache: &token_cache,
                            notifier: &notifier,
                            scheduler: &scheduler,
                            add_queue: &add_queue,
                            remove_queue: &remove_queue,
                        };
                        let stop_requested = Self::run_cycle(&ctx, Some(&stop_rx));
                        if stop_requested {
                            info!(
                                "Playlist sync scheduler received shutdown signal during phase gap"
                            );
                            break;
                        }
                    }
                }
            }
        });

        Self {
            stop_tx: Mutex::new(Some(stop_tx)),
            handle: Mutex::new(Some(handle)),
        }
    }

    pub(crate) fn shutdown(&self) {
        if let Some(tx) = self.stop_tx.lock().map(|mut g| g.take()).unwrap_or(None) {
            let _ = tx.send(());
        }

        if let Some(handle) = self.handle.lock().map(|mut g| g.take()).unwrap_or(None)
            && let Err(e) = handle.join()
        {
            error!(?e, "Failed to join playlist sync scheduler thread");
        }
    }

    fn run_cycle(ctx: &CycleContext<'_>, stop_rx: Option<&mpsc::Receiver<()>>) -> bool {
        Self::process_queue(
            ctx.base_url,
            ctx.paths,
            ctx.token_cache,
            ctx.notifier,
            ctx.scheduler,
            ctx.add_queue,
            true,
        );

        if let Some(stop_rx) = stop_rx {
            match stop_rx.recv_timeout(PHASE_GAP) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    Self::process_queue(
                        ctx.base_url,
                        ctx.paths,
                        ctx.token_cache,
                        ctx.notifier,
                        ctx.scheduler,
                        ctx.remove_queue,
                        false,
                    );
                    return true;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
        }

        Self::process_queue(
            ctx.base_url,
            ctx.paths,
            ctx.token_cache,
            ctx.notifier,
            ctx.scheduler,
            ctx.remove_queue,
            false,
        );
        false
    }

    fn drain_queue(queue: &Arc<Mutex<QueueMap>>) -> QueueMap {
        let mut guard = match queue.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        std::mem::take(&mut *guard)
    }

    fn merge_back(queue: &Arc<Mutex<QueueMap>>, failed: QueueMap) {
        if failed.is_empty() {
            return;
        }
        let mut guard = match queue.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        for (target, uris) in failed {
            let entry = guard.entry(target).or_default();
            for uri in uris {
                entry.insert(uri);
            }
        }
    }

    fn process_queue(
        base_url: &str,
        paths: &HashMap<SpotifyApiAction, String>,
        token_cache: &Arc<dyn TokenCache>,
        notifier: &Arc<dyn ErrorNotification>,
        scheduler: &Arc<RequestScheduler>,
        queue: &Arc<Mutex<QueueMap>>,
        is_add: bool,
    ) {
        let drained = Self::drain_queue(queue);
        if drained.is_empty() {
            return;
        }
        let mut failed = HashMap::new();
        for (target, uris) in drained {
            let result = if is_add {
                Self::run_add_batch(base_url, paths, token_cache, scheduler, &target, &uris)
            } else {
                Self::run_remove_batch(base_url, paths, token_cache, scheduler, &target, &uris)
            };
            if let Err(e) = result {
                error!(error = %e, ?target, "Failed to process playlist queue batch");
                notifier.notify(&e.to_string());
                failed.insert(target, uris);
            }
        }
        Self::merge_back(queue, failed);
    }

    fn run_add_batch(
        base_url: &str,
        paths: &HashMap<SpotifyApiAction, String>,
        token_cache: &Arc<dyn TokenCache>,
        scheduler: &Arc<RequestScheduler>,
        target: &QueueTarget,
        uris: &IndexSet<String>,
    ) -> AppResult<()> {
        if uris.is_empty() {
            return Ok(());
        }
        let token = Self::token_from_cache(token_cache)?;
        let ordered_uris: Vec<String> = uris.iter().cloned().collect();
        match target {
            QueueTarget::Liked => {
                let url = format!(
                    "{}{}?uris={}",
                    base_url,
                    Self::path(paths, SpotifyApiAction::Library)?,
                    ordered_uris.join(",")
                );
                scheduler.run("cron add to liked", ScheduleMode::Wait, || {
                    ureq::put(&url)
                        .set("Authorization", &format!("Bearer {token}"))
                        .call()
                })?;
            }
            QueueTarget::Playlist(playlist_id) => {
                let path = Self::path(paths, SpotifyApiAction::PlaylistItems)?
                    .replace("{id}", playlist_id);
                let url = format!("{base_url}{path}");
                let payload_uris = ordered_uris.clone();
                scheduler.run("cron add to playlist", ScheduleMode::Wait, || {
                    ureq::post(&url)
                        .set("Authorization", &format!("Bearer {token}"))
                        .set("Content-Type", "application/json")
                        .send_json(ureq::json!({ "uris": payload_uris, "position": 0 }))
                })?;
            }
        }
        Ok(())
    }

    fn run_remove_batch(
        base_url: &str,
        paths: &HashMap<SpotifyApiAction, String>,
        token_cache: &Arc<dyn TokenCache>,
        scheduler: &Arc<RequestScheduler>,
        target: &QueueTarget,
        uris: &IndexSet<String>,
    ) -> AppResult<()> {
        if uris.is_empty() {
            return Ok(());
        }
        let token = Self::token_from_cache(token_cache)?;
        let ordered_uris: Vec<String> = uris.iter().cloned().collect();
        match target {
            QueueTarget::Liked => {
                let url = format!(
                    "{}{}?uris={}",
                    base_url,
                    Self::path(paths, SpotifyApiAction::Library)?,
                    ordered_uris.join(",")
                );
                scheduler.run("cron remove from liked", ScheduleMode::Wait, || {
                    ureq::request("DELETE", &url)
                        .set("Authorization", &format!("Bearer {token}"))
                        .call()
                })?;
            }
            QueueTarget::Playlist(playlist_id) => {
                let path = Self::path(paths, SpotifyApiAction::PlaylistItems)?
                    .replace("{id}", playlist_id);
                let url = format!("{base_url}{path}");
                let tracks: Vec<_> = ordered_uris
                    .iter()
                    .map(|u| ureq::json!({ "uri": u }))
                    .collect();
                scheduler.run("cron remove from playlist", ScheduleMode::Wait, || {
                    ureq::request("DELETE", &url)
                        .set("Authorization", &format!("Bearer {token}"))
                        .set("Content-Type", "application/json")
                        .send_json(ureq::json!({ "items": tracks }))
                })?;
            }
        }
        Ok(())
    }

    fn path(
        paths: &HashMap<SpotifyApiAction, String>,
        action: SpotifyApiAction,
    ) -> AppResult<String> {
        paths
            .get(&action)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No path configured for action '{action:?}'").into())
    }

    fn token_from_cache(token_cache: &Arc<dyn TokenCache>) -> AppResult<String> {
        token_cache
            .load()
            .ok_or_else(|| anyhow::anyhow!("No access token available").into())
    }
}

impl Drop for PlaylistSyncScheduler {
    fn drop(&mut self) {
        self.shutdown();
    }
}
