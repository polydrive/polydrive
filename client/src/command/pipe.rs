use crate::command::Command;
use crate::CommandHandler;
use anyhow::{anyhow, Result};
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use log::{error, info};
use std::fs::remove_file;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub struct CommandListener {
    /// The socket listener
    listener: LocalSocketListener,
    command_handler: CommandHandler,
}

fn handle_error(conn: std::io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
    match conn {
        Ok(val) => Some(val),
        Err(error) => {
            eprintln!("Incoming connection failed: {}", error);
            None
        }
    }
}

impl CommandListener {
    pub fn new(socket: &str, command_handler: CommandHandler) -> Result<Self> {
        let socket_path = PathBuf::from(socket);
        if socket_path.exists() {
            if let Err(e) = remove_file(&socket_path) {
                error!(
                    "failed to remove existing socket file. path={}, details={}",
                    socket,
                    e.to_string()
                );
            }
        }

        let listener = LocalSocketListener::bind(socket).map_err(|e| {
            anyhow!(
                "failed to bind socket on path. path={}, details={}",
                socket,
                e.to_string()
            )
        })?;
        Ok(Self {
            listener,
            command_handler,
        })
    }

    pub async fn listen(&self) -> Result<()> {
        info!("waiting for commands");

        for stream in self.listener.incoming().filter_map(handle_error) {
            // Read the message received by the client and parse the command
            let mut reader = BufReader::new(stream);
            let mut raw = String::new();
            reader.read_line(&mut raw)?;

            // Execute the command
            let command = Command::from(raw.trim());
            let response = self.command_handler.execute(command).await?;

            // Send the command response to the client
            let mut writer = BufWriter::new(reader.get_mut());
            writer.write_all(response.as_bytes())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct CommandWriter {
    /// The stream where to write data
    stream: LocalSocketStream,
}

impl CommandWriter {
    pub fn new(socket: &str) -> Result<Self> {
        if !PathBuf::from(socket).exists() {
            return Err(anyhow!("cannot establish a connection with the daemon, please ensure a daemon is running on the host"));
        }

        let stream = LocalSocketStream::connect(socket).map_err(|e| {
            anyhow!(
                "failed to connect to socket on path. path={}, details={}",
                socket,
                e.to_string()
            )
        })?;
        Ok(Self { stream })
    }

    // Send a command onto the pipe, and wait for the response
    pub fn send(self, command: Command) -> Result<String> {
        let data: &str = command.into();

        // Send the command to the socket server
        let mut conn = self.stream;
        conn.write_all(format!("{}\n", data).as_bytes())?;

        // Read the response
        let mut conn = BufReader::new(conn);
        let mut buffer = Vec::<u8>::new();
        conn.read_to_end(&mut buffer)?;

        Ok(String::from_utf8(buffer)?)
    }
}
