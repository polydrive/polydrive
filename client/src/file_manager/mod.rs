use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Read};
use log::info;
use reqwest::{Client};

pub struct FileManager {
    client: Client
}

impl FileManager {
    pub fn init() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }

    pub async fn upload(&self, url: &str, file: File, filename: &OsStr) -> Result<(), reqwest::Error> {
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        self.client.put(url).body(buffer).send().await?;
        info!("{:?} successfully uploaded", filename);
        Ok(())
    }

    /*pub fn download() {
        return
    }*/
}