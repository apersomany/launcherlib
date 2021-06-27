use std::{error::Error, fmt};
use reqwest::{Client, Response};
use tokio::fs::{create_dir_all, metadata, read, write};

#[derive(Debug)]
pub struct ContextError {}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContextError")
    }
}

impl Error for ContextError {}

pub struct Context { 
    pub path: String,
    pub client: Client,
    pub max_concurrent: u8
}

impl Context {
    pub async fn new(path: &str) -> Result<Self, ContextError> {
        let ctx = Self { 
            path: path.to_string(),
            client: Client::new(),
            max_concurrent: 16
        };
        Self::ensure_dir(&ctx, "").await?;
        Ok(ctx)
    }

    pub async fn default() -> Result<Self, ContextError> {
        Self::new("default").await
    }

    pub fn path(&self, path: &str) -> String { format!("{}/{}", self.path, path) }

    pub async fn get(&self, url: &str) -> reqwest::Result<Response> {
        self.client.get(url).send().await
    }

    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>, ContextError> {
        match read(Self::path(&self, path)).await {
            Ok(b) => Ok(b),
            Err(_) => Err(ContextError {})
        }
    }

    pub async fn ensure_dir(&self, path: &str) -> Result<(), ContextError> {
        let path = Self::path(&self, path);
        match metadata(&path).await {
            Ok(m) => {
                if m.is_dir() { Ok(()) }
                else { Err(ContextError {}) }}, 
            Err(_) => {
                if create_dir_all(&path).await.is_ok() { Ok(()) } 
                else { Err(ContextError {}) }
            }
        }
    }

    pub async fn write_file(&self, path: &str, bytes: &[u8]) -> Result<(), ContextError> {
        let mut split: Vec<&str> = path.split("/").collect();
        if split.pop().is_some() {
            let dir_path = split.join("/");
            Self::ensure_dir(&self, &dir_path).await?;
            if write(Self::path(&self, path), bytes).await.is_ok() { Ok(()) }
            else { Err(ContextError {}) }
        } else { Err(ContextError {}) }
    }

    pub async fn check_file_size(&self, path: &str, size: u64) -> Result<(), ContextError> {
        match metadata(Self::path(&self, path)).await {
            Ok(m) => {
                if m.len() == size { Ok(()) } 
                else { Err(ContextError {}) }
            },
            Err(_) => Err(ContextError {})
        }
    }
}