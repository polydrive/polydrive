use crate::grpc::server::file_manager_service_client::FileManagerServiceClient;
use crate::grpc::upload::{UploadEvent, UploadStatus};
use anyhow::Result;
use log::{debug, error, info};
use reqwest::Client;
use std::fs::{create_dir_all, File};
use std::io::copy;
use std::path::PathBuf;
use tonic::transport::Channel;

#[derive(Clone)]
pub struct StorageManager {
    http_client: Client,
    grpc_client: FileManagerServiceClient<Channel>,
}

impl StorageManager {
    /// Init a reqwest client to make HTTP calls
    pub fn init(grpc_client: FileManagerServiceClient<Channel>) -> Self {
        let http_client = reqwest::Client::new();
        Self {
            http_client,
            grpc_client,
        }
    }

    /// Upload a file to an URL and notify
    /// the remote server with an `UploadEvent`.
    pub async fn upload(&self, url: &str, path: &str, _file: File) -> Result<()> {
        debug!("reading file content in bytes. file={:?}", path);
        let buffer = std::fs::read(path)?;

        let (status, message): (UploadStatus, Option<String>) =
            match self.http_client.put(url).body(buffer).send().await {
                Ok(res) if res.status() == 200 => (UploadStatus::Success, None),
                Ok(res) => {
                    error!("Server responded with status code {}", res.status());
                    (UploadStatus::Failure, Some(res.text().await?))
                }
                Err(e) => {
                    error!("failed to upload file. details = {}", e.to_string());
                    (UploadStatus::Failure, Some(e.to_string()))
                }
            };

        info!("successfully uploaded file {}", path);
        info!("status: {:#?}, message: {:#?}", status, message);

        self.notify(UploadEvent {
            path: path.to_string(),
            status: status.into(),
            message,
        })
        .await?;

        Ok(())
    }

    /// Download a file from minio through a presigned URL
    #[allow(dead_code)]
    pub async fn download(&self, url: &str, path: &str) -> Result<()> {
        let resp = self.http_client.get(url).send().await?.text().await?;
        let path_buf = PathBuf::from(path);
        create_dir_all(path_buf.parent().unwrap())?;
        let mut out = File::create(path).expect("failed to create file");
        copy(&mut resp.as_bytes(), &mut out).expect("failed to copy content");
        Ok(())
    }

    /// Notify the remote server with an `UploadEvent`
    async fn notify(&self, event: UploadEvent) -> Result<()> {
        debug!(
            "notifying remote server of an upload event. event={:?}",
            event
        );
        self.grpc_client.clone().on_upload_event(event).await?;
        Ok(())
    }
}
