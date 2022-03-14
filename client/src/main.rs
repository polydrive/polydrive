mod agent;
mod server;

use agent::list::ListCommand;
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use log::LevelFilter;
use server::watch::WatchCommand;

pub trait Handler {
    /// Executes the command handler.
    ///
    /// Every command should take no argument, has it is built at runtime with these arguments.
    /// Also, a command must always return a `Result<()>`.
    fn handler(&self) -> Result<()>;
}

#[derive(Parser, Debug)]
#[clap(version)]
struct Cli {
    /// The level of verbosity.
    #[clap(short, long, parse(from_occurrences))]
    pub(crate) verbose: usize,

    /// If set, the client will be act as a server.
    #[clap(short, long)]
    server: bool,

    /// The command to execute.
    #[clap(subcommand)]
    command: Command,
}

impl Cli {
    /// Get the current command to execute.
    ///
    /// If the command is not valid for the current enabled mode (server or agent), we must throw an error.
    pub fn command(self) -> Result<Box<dyn Handler>> {
        let mode = if self.server { "server" } else { "agent" };
        let command = if self.server {
            self.get_server_commands()
        } else {
            self.get_agent_commands()
        };

        if command.is_none() {
            return Err(anyhow!("Invalid command. Check if the command exists and if it is valid for current mode : {}", mode));
        }

        Ok(command.unwrap())
    }

    /// Get the agent mode commands
    ///
    /// Must return `None` for every commands that is not compatible with the current mode.
    pub fn get_agent_commands(self) -> Option<Box<dyn Handler>> {
        match self.command {
            Command::List(cmd) => Some(Box::new(cmd)),
            _ => None,
        }
    }

    /// Get the server mode commands
    ///
    /// Must return `None` for every commands that is not compatible with the current mode.
    pub fn get_server_commands(self) -> Option<Box<dyn Handler>> {
        match self.command {
            Command::Watch(cmd) => Some(Box::new(cmd)),
            _ => None,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    List(ListCommand),
    Watch(WatchCommand),
}

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    // Configure the logger
    env_logger::Builder::new()
        .filter_level(match cli.verbose {
            1 => LevelFilter::Debug,
            2 => LevelFilter::Trace,
            _ => LevelFilter::Info,
        })
        .init();

    cli.command()?.handler()
}
