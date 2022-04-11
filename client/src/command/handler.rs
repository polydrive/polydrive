use crate::command::Command;
use crate::grpc::server::file_manager_service_client::FileManagerServiceClient;
use anyhow::Result;
use log::info;
use prettytable::{cell, format, row, Table};
use std::path::PathBuf;
use tonic::transport::Channel;

/// The `CommandHandler` is responsible of handling commands
/// that cames from the client CLI.
#[derive(Debug)]
pub struct CommandHandler {
    #[allow(dead_code)]
    client: FileManagerServiceClient<Channel>,
}

impl CommandHandler {
    pub fn new(client: FileManagerServiceClient<Channel>) -> Self {
        Self { client }
    }

    /// Execute the command supplied in arguments and return it's output.
    pub async fn execute(&self, command: Command) -> Result<String> {
        match command {
            Command::ListFiles => self.list().await,
            _ => Ok(String::from("command not found")),
        }
    }

    /// List the files indexed
    pub async fn list(&self) -> Result<String> {
        info!("getting files from server");
        // Create the table
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_CLEAN);
        table.add_row(row!["FILENAME", "PATH", "VERSION", "SYNCED"]);

        let response = self.client.clone().get_files(()).await?.into_inner();
        for file in response.data {
            let is_sync = PathBuf::from(&file.path).exists();
            table.add_row(row![
                file.base_name,
                file.path,
                file.version.unwrap_or(1),
                is_sync
            ]);
        }

        Ok(table.to_string())
    }
}
