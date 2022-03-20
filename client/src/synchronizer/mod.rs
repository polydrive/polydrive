use crate::grpc::file_manager_service_client::FileManagerServiceClient;
use crate::grpc::Notification;
use anyhow::Result;
use log::{debug, info};
use tonic::Streaming;

/// The `Synchronizer` component is responsible to subscribe to
/// remote server notifications and synchronize the fs with the remote fs.
pub struct Synchronizer {
    stream: Streaming<Notification>,
}

impl Synchronizer {
    /// Bootstrap the synchronizer by opening connection
    /// to the server stream.
    pub async fn bootstrap(server_url: String) -> Result<Self> {
        debug!("boostraping synchronizer");

        debug!("subscribe to notifications stream");
        let mut client = FileManagerServiceClient::connect(server_url).await?;
        let stream = client.subscribe_notification(()).await?.into_inner();

        Ok(Self { stream })
    }

    /// Listen for notifications.
    ///
    /// This is a blocking method.
    pub async fn listen(&mut self) -> Result<()> {
        info!("starting synchronizer");

        while let Some(notification) = &self.stream.message().await? {
            info!("received notification = {:?}", notification)
        }

        Ok(())
    }
}
