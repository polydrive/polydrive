use crate::{Handler, Result};
use clap::Args;

/// Watch files on the host
#[derive(Debug, Args)]
pub struct WatchCommand;

impl Handler for WatchCommand {
    fn handler(&self) -> Result<()> {
        log::info!("This should watch the files");

        Ok(())
    }
}
