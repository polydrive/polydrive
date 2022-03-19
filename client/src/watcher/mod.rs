mod pool;

use crate::watcher::pool::Pool;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{debug, info};
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[async_trait]
pub trait WatcherListener {
    /// Function called whenever a file is created
    async fn on_event(&self, event: &DebouncedEvent) -> Result<()>;
}

#[derive(Clone)]
pub struct PoolWatcher {
    /// The pool of files to watch
    pub(crate) pool: Pool,

    /// Listener suscribed to the file watch events
    pub(crate) listeners: Vec<Arc<Mutex<dyn WatcherListener>>>,
}

impl PoolWatcher {
    /// Init a `Watcher` instance
    pub fn init(paths: &[String]) -> Self {
        let pool = Pool::from(&paths.to_vec());
        let listeners = vec![];
        Self { pool, listeners }
    }

    /// Run the watcher to watch files.
    pub async fn start(&self) -> Result<()> {
        info!("configuring sender and receiver on channel for events");
        let (tx, rx) = std::sync::mpsc::channel();

        // TODO: make watch time configurable?
        let mut watcher = notify::watcher(tx, Duration::from_secs(2))
            .map_err(|e| anyhow!("failed to create watcher with error={}", e))?;

        // Add each path to the watcher
        for path in &self.pool.paths {
            debug!("configuring watcher for path={}", &path.display());
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| anyhow!("failed to watch path={}, reason={}", &path.display(), e))?;
        }

        info!("successfully configured watchers, waiting for events");

        loop {
            match rx.recv() {
                Ok(event) => self.notify(&event).await?,
                Err(e) => println!("received error from channel: {:?}", e),
            }
        }
    }

    pub async fn notify(&self, event: &DebouncedEvent) -> Result<()> {
        debug!(
            "notifying {} listeners for event={:?}",
            &self.listeners.len(),
            event
        );
        for locked_listener in self.listeners.clone() {
            let listener = locked_listener.lock().unwrap();
            // TODO: notify in parallel?
            listener.on_event(event).await?;
        }
        Ok(())
    }

    pub fn add_listener(&mut self, listener: Arc<Mutex<dyn WatcherListener>>) -> &mut Self {
        debug!("adding listener");
        self.listeners.push(listener);
        self
    }
}
