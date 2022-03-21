mod agent;
mod config;
mod grpc;
mod indexer;
mod synchronizer;
mod watcher;

use crate::config::Config;
use crate::indexer::Indexer;
use crate::synchronizer::Synchronizer;
use crate::watcher::PoolWatcher;
use agent::list::ListCommand;
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use log::{info, LevelFilter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
    daemon: bool,

    /// A list of files or directories to watch.
    ///
    /// Supports glob-based path, e.g: /tmp/**/**.png. If the path is a glob, it'll be expanded, so /tmp/*/**.png will
    /// detect every png FILE present behind your /tmp folder. Be aware, if you pass a glob path, it will not watch folders,
    /// but only existing files matching the glob pattern when the command is executed.
    ///
    /// You can use the client mode to add more watch later.
    ///
    /// Examples:
    ///
    /// To watch every changes, files and folders, inside /tmp :
    ///
    /// client --server --watch /tmp
    ///
    /// To watch every changes only on existing .png files :
    ///
    /// client --server --watch /**/*.png
    #[clap(long = "watch")]
    files: Vec<String>,

    /// A path to a configuration file in .yml format.
    ///
    /// Example:
    ///
    /// client --server --watch /tmp --config /tmp/polydrived.yml
    #[clap(short, long)]
    config: Option<PathBuf>,

    /// The command to execute.
    #[clap(subcommand)]
    command: Option<Command>,
}

impl Cli {
    /// Get the current command to execute.
    ///
    /// If the command is not valid for the current enabled mode (server or agent), we must throw an error.
    pub fn command(self) -> Result<Box<dyn Handler>> {
        if let Some(command) = self.command {
            return match command {
                Command::List(cmd) => Ok(Box::new(cmd)),
            };
        }

        Err(anyhow!(
            "no command provided. To start client in server mode, use --server."
        ))
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    List(ListCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    // Configure the logger
    env_logger::Builder::new()
        .filter_level(match cli.verbose {
            1 => LevelFilter::Debug,
            2 => LevelFilter::Trace,
            _ => LevelFilter::Info,
        })
        .init();

    if cli.daemon {
        let config = Config::load(cli.config.clone(), Some(true))?;

        info!("starting daemon");

        let indexer = Indexer::bootstrap(&config.get_server_address()).await?;

        // Start synchronizer into another thread because
        // PoolWatcher start() method is blocking.
        tokio::task::spawn(async move {
            Synchronizer::bootstrap(&config.clone().get_server_address())
                .await?
                .listen()
                .await
        });

        PoolWatcher::init(&cli.files)
            .add_listener(Arc::new(Mutex::new(indexer.clone())))
            .start()
            .await?;

        return Err(anyhow!("failed to run the daemon"));
    }

    cli.command()?.handler()
}
