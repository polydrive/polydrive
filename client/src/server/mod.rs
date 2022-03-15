use crate::server::watcher::PoolWatcher;
use anyhow::Result;
use log::info;

mod watcher;

pub struct Server;

impl Server {
    /// Start the server
    pub fn start(paths: &[String]) -> Result<()> {
        info!("starting server");

        let watcher = PoolWatcher::init(paths);

        watcher.start()
    }
}
