use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, copy, Read};
use log::info;
use reqwest::{Client};

pub struct FileManager {
    client: Client
}

impl FileManager {
    /// Init a reqwest client to make HTTP calls
    pub fn init() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }

    /// Upload a file to minio through a presigned URL
    pub async fn upload(&self, url: &str, file: File, filename: &OsStr) -> Result<(), reqwest::Error> {
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        self.client.put(url).body(buffer).send().await?;
        info!("{:?} successfully uploaded", filename);
        Ok(())
    }

    /// Download a file from minio through a presigned URL
    pub async fn download(&self, url: &str, path: &str) -> Result<(), reqwest::Error> {
        let mut resp = self.client.get(url).send().await?;
        let mut out = File::create(path).expect("failed to create file");
        copy(&mut resp, &mut out).expect("failed to copy content");
        Ok(())
    }
}