use crate::{Handler, Result};
use clap::Args;

/// List the files synchronized in the system
#[derive(Debug, Args)]
pub struct ListCommand;

impl Handler for ListCommand {
    fn handler(&self) -> Result<()> {
        log::info!("This should print the files");

        Ok(())
    }
}
