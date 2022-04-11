use crate::command::Command;
use crate::{CommandWriter, Handler, Result};
use clap::Args;

/// List the files synchronized in the system
#[derive(Debug, Args)]
pub struct ListCommand;

impl Handler for ListCommand {
    fn handler(&self, command_bus: CommandWriter) -> Result<()> {
        let response = command_bus.send(Command::ListFiles)?;
        println!("{}", response);
        Ok(())
    }
}
