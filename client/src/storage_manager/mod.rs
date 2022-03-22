use anyhow::Result;
use log::info;
use reqwest::Client;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{copy, BufReader, Read};

#[derive(Clone)]
pub struct StorageManager {
    client: Client,
}

impl StorageManager {
    /// Init a reqwest client to make HTTP calls
    pub fn init() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }

    /// Upload a file to minio through a presigned URL
    pub async fn upload(&self, url: &str, file: File, filename: &OsStr) -> Result<()> {
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        self.client.put(url).body(buffer).send().await?;
        info!("{:?} successfully uploaded", filename);
        Ok(())
    }

    /// Download a file from minio through a presigned URL
    #[allow(dead_code)]
    pub async fn download(&self, url: &str, path: &str) -> Result<()> {
        let resp = self.client.get(url).send().await?.text().await?;
        let mut out = File::create(path).expect("failed to create file");
        copy(&mut resp.as_bytes(), &mut out).expect("failed to copy content");
        Ok(())
    }
}
