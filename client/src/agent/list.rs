use crate::{Handler, Result};
use clap::Args;
use interprocess::local_socket::LocalSocketStream;
use std::io::{prelude::*, BufReader};

/// List the files synchronized in the system
#[derive(Debug, Args)]
pub struct ListCommand;

impl Handler for ListCommand {
    fn handler(&self) -> Result<()> {
        log::info!("This should print the files");

        let mut conn = LocalSocketStream::connect("/tmp/example.sock")?;
        conn.write_all(b"list")?;

        let mut conn = BufReader::new(conn);
        let mut buffer = String::new();
        conn.read_line(&mut buffer)?;

        println!("Server answered: {}", buffer);

        Ok(())
    }
}
