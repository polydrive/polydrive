mod pool;

use crate::server::watcher::pool::Pool;
use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::time::Duration;

pub struct PoolWatcher {
    /// The pool of files to watch
    pub(crate) pool: Pool,
}

impl PoolWatcher {
    /// Init a `Watcher` instance
    pub fn init(paths: &[String]) -> Self {
        let pool = Pool::from(&paths.to_vec());
        Self { pool }
    }

    /// Run the watcher to watch files.
    pub fn start(&self) -> Result<()> {
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
                Ok(event) => self.handle_event(event),
                Err(e) => println!("received error from channel: {:?}", e),
            }
        }
    }

    /// Handle events emitted by the watcher.
    fn handle_event(&self, event: DebouncedEvent) {
        match event {
            DebouncedEvent::Create(path) => debug!("new file detected. file={}", &path.display()),
            DebouncedEvent::Write(path) => debug!("modification detected. file={}", &path.display()),
            DebouncedEvent::Chmod(path) => debug!("file attributes updated. file={}", &path.display()),
            DebouncedEvent::Remove(path) => debug!("removed file. file={}", &path.display()),
            DebouncedEvent::Rename(old, new) => debug!("file renamed. old={}, new={}", &old.display(), &new.display()),
            DebouncedEvent::Rescan => warn!("a problem has been detected that makes it necessary to re-scan the watched directories."),
            DebouncedEvent::Error(e, path) => {
                if let Some(path) = path {
                    error!("an error occurred on path={}. details={}", &path.display(), e)
                } else {
                    error!("an error occurred. details={}", e)
                }
            },
            _ => debug!("ignoring event = {:?} ", event)
        }
    }
}
