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

        let mut stream = LocalSocketStream::connect("/tmp/polydrive.sock")?;
        stream.write_all(b"list\n")?;
        println!("List command sent");

        let mut reader = BufReader::new(stream);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        println!(
            "Server answered: {}",
            String::from_utf8(buffer).expect("failed to convert buffer to string")
        );

        Ok(())
    }
}
