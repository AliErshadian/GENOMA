use std::path::{Path, PathBuf};

use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

#[derive(Clone, Debug)]
pub struct FsBlobStore {
    root: PathBuf,
}

impl FsBlobStore {
    pub async fn new(root: PathBuf) -> ApiResult<Self> {
        fs::create_dir_all(&root)
            .await
            .map_err(|err| ApiError::internal(format!("blob dir: {err}")))?;
        Ok(Self { root })
    }

    pub fn path_for(&self, key: &str) -> PathBuf {
        self.root.join(key)
    }

    pub async fn put_stream<S, E>(&self, key: &str, mut stream: S) -> ApiResult<u64>
    where
        S: futures_util::Stream<Item = Result<bytes::Bytes, E>> + Unpin,
        E: std::fmt::Display,
    {
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| ApiError::internal(err.to_string()))?;
        }
        let mut file = fs::File::create(&path)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        let mut written = 0_u64;
        while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
            let chunk = chunk.map_err(|err| ApiError::internal(err.to_string()))?;
            written += chunk.len() as u64;
            file.write_all(&chunk)
                .await
                .map_err(|err| ApiError::internal(err.to_string()))?;
        }
        file.flush()
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        Ok(written)
    }

    pub async fn put_bytes(&self, key: &str, bytes: &[u8]) -> ApiResult<()> {
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| ApiError::internal(err.to_string()))?;
        }
        fs::write(path, bytes)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))
    }

    pub async fn put_field(
        &self,
        key: &str,
        mut field: axum::extract::multipart::Field<'_>,
        max_bytes: u64,
    ) -> ApiResult<u64> {
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| ApiError::internal(err.to_string()))?;
        }
        let mut file = fs::File::create(&path)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        let mut written = 0_u64;
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|err| ApiError::bad_request(err.to_string()))?
        {
            written += chunk.len() as u64;
            if written > max_bytes {
                drop(file);
                let _ = fs::remove_file(&path).await;
                return Err(ApiError::payload_too_large(format!(
                    "file exceeds maximum size of {max_bytes} bytes"
                )));
            }
            file.write_all(&chunk)
                .await
                .map_err(|err| ApiError::internal(err.to_string()))?;
        }
        file.flush()
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        Ok(written)
    }

    pub fn local_path(&self, key: &str) -> PathBuf {
        self.path_for(key)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

pub fn object_key(analysis_id: Uuid, filename: &str) -> String {
    let safe: String = filename
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    format!("{analysis_id}/{safe}")
}
