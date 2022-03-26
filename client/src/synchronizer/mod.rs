use crate::grpc::file::FileRequest;
use crate::grpc::server::file_manager_service_client::FileManagerServiceClient;
use crate::grpc::server::Notification;
use crate::storage_manager::StorageManager;
use anyhow::Result;
use log::{debug, info};
use std::path::PathBuf;
use tonic::transport::Channel;
use tonic::Streaming;

/// The `Synchronizer` component is responsible to subscribe to
/// remote server notifications and synchronize the fs with the remote fs.
pub struct Synchronizer {
    client: FileManagerServiceClient<Channel>,
    stream: Streaming<Notification>,
    storage_manager: StorageManager,
}

impl Synchronizer {
    /// Bootstrap the synchronizer by opening connection
    /// to the server stream.
    pub async fn bootstrap(server_url: &str) -> Result<Self> {
        debug!("boostraping synchronizer");

        let mut client = FileManagerServiceClient::connect(server_url.to_string()).await?;

        debug!("subscribe to notifications stream");
        let stream = client.subscribe_notification(()).await?.into_inner();
        let storage_manager = StorageManager::init(client.clone());

        Ok(Self {
            stream,
            client,
            storage_manager,
        })
    }

    /// Listen for notifications.
    ///
    /// This is a blocking method.
    pub async fn listen(&mut self) -> Result<()> {
        info!("starting synchronizer");

        while let Some(notification) = &self.stream.message().await? {
            debug!("received notification = {:?}", notification);

            if let Some(file) = &notification.file {
                // At the moment, as we don't have the versioning enabled right now,
                // we simply check if the file exists on the host, and if not, we request a new
                // download link in order to download the file.
                if PathBuf::from(&file.path).exists() {
                    info!(
                        "file {} already exists. no synchronization needed.",
                        &file.path
                    );
                    continue;
                }

                info!("synchronization required due to new file detected that is not present on disk. file={}", &file.path);

                let response = self
                    .client
                    .clone()
                    .file(FileRequest {
                        client_name: None,
                        path: file.path.to_string(),
                        version: Some(1.to_string()),
                    })
                    .await?
                    .into_inner();

                self.storage_manager
                    .download(&response.link, &file.path)
                    .await?;

                info!("successfully synchronized file. file={}", &file.path)
            }
        }

        Ok(())
    }
}
