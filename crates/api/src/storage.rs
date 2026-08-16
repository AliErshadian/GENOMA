use std::path::{Path, PathBuf};
use std::sync::Arc;

use aws_credential_types::Credentials;
use aws_sdk_s3::config::{BehaviorVersion, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::config::{AppConfig, StorageBackend};
use crate::error::{ApiError, ApiResult};

/// Object storage for uploaded analysis inputs.
///
/// Local `fs` is the default. `s3` targets MinIO or any S3-compatible endpoint.
#[derive(Clone, Debug)]
pub enum BlobStore {
    Fs(FsBlobStore),
    S3(S3BlobStore),
}

impl BlobStore {
    pub async fn from_config(config: &AppConfig) -> ApiResult<Self> {
        match config.storage_backend {
            StorageBackend::Fs => Ok(Self::Fs(FsBlobStore::new(config.blob_dir.clone()).await?)),
            StorageBackend::S3 => Ok(Self::S3(S3BlobStore::from_config(config).await?)),
        }
    }

    pub fn backend_name(&self) -> &'static str {
        match self {
            Self::Fs(_) => "fs",
            Self::S3(_) => "s3",
        }
    }

    pub async fn put_bytes(&self, key: &str, bytes: &[u8]) -> ApiResult<()> {
        match self {
            Self::Fs(store) => store.put_bytes(key, bytes).await,
            Self::S3(store) => store.put_bytes(key, bytes).await,
        }
    }

    pub async fn put_field(
        &self,
        key: &str,
        field: axum::extract::multipart::Field<'_>,
        max_bytes: u64,
    ) -> ApiResult<u64> {
        match self {
            Self::Fs(store) => store.put_field(key, field, max_bytes).await,
            Self::S3(store) => store.put_field(key, field, max_bytes).await,
        }
    }

    /// Returns a filesystem path the analysis job can open.
    /// For S3, downloads into a local cache under `blob_dir` when needed.
    pub async fn local_path_or_download(&self, key: &str) -> ApiResult<PathBuf> {
        match self {
            Self::Fs(store) => Ok(store.local_path(key)),
            Self::S3(store) => store.local_path_or_download(key).await,
        }
    }
}

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

#[derive(Clone, Debug)]
pub struct S3BlobStore {
    client: S3Client,
    bucket: String,
    cache_root: PathBuf,
}

impl S3BlobStore {
    pub async fn from_config(config: &AppConfig) -> ApiResult<Self> {
        let s3 = config.s3.as_ref().ok_or_else(|| {
            ApiError::internal(
                "GENOMA_STORAGE_BACKEND=s3 requires S3_ENDPOINT, S3_BUCKET, S3_ACCESS_KEY, S3_SECRET_KEY",
            )
        })?;
        Self::new(
            &s3.endpoint,
            &s3.region,
            &s3.bucket,
            &s3.access_key,
            &s3.secret_key,
            s3.use_path_style,
            config.blob_dir.clone(),
        )
        .await
    }

    pub async fn new(
        endpoint: &str,
        region: &str,
        bucket: &str,
        access_key: &str,
        secret_key: &str,
        use_path_style: bool,
        cache_root: PathBuf,
    ) -> ApiResult<Self> {
        fs::create_dir_all(&cache_root)
            .await
            .map_err(|err| ApiError::internal(format!("blob cache dir: {err}")))?;

        let credentials = Credentials::new(access_key, secret_key, None, None, "genoma-env");
        let mut builder = aws_sdk_s3::config::Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(region.to_string()))
            .credentials_provider(credentials)
            .endpoint_url(endpoint);
        if use_path_style {
            builder = builder.force_path_style(true);
        }
        let client = S3Client::from_conf(builder.build());

        Ok(Self {
            client,
            bucket: bucket.to_string(),
            cache_root,
        })
    }

    fn cache_path(&self, key: &str) -> PathBuf {
        self.cache_root.join("_s3_cache").join(key)
    }

    pub async fn put_bytes(&self, key: &str, bytes: &[u8]) -> ApiResult<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(bytes.to_vec()))
            .send()
            .await
            .map_err(|err| ApiError::internal(format!("s3 put: {err}")))?;
        Ok(())
    }

    pub async fn put_field(
        &self,
        key: &str,
        mut field: axum::extract::multipart::Field<'_>,
        max_bytes: u64,
    ) -> ApiResult<u64> {
        // Stream to a temp file first so large uploads stay bounded-memory,
        // then push the object to S3.
        let staging = self.cache_root.join(format!("_upload_{}", Uuid::new_v4()));
        if let Some(parent) = staging.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| ApiError::internal(err.to_string()))?;
        }
        let mut file = fs::File::create(&staging)
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
                let _ = fs::remove_file(&staging).await;
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
        drop(file);

        let body = ByteStream::from_path(&staging)
            .await
            .map_err(|err| ApiError::internal(format!("s3 staging read: {err}")))?;
        let result = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .send()
            .await;
        let _ = fs::remove_file(&staging).await;
        result.map_err(|err| ApiError::internal(format!("s3 put: {err}")))?;
        Ok(written)
    }

    pub async fn local_path_or_download(&self, key: &str) -> ApiResult<PathBuf> {
        let path = self.cache_path(key);
        if path.exists() {
            return Ok(path);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| ApiError::internal(err.to_string()))?;
        }
        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|err| ApiError::internal(format!("s3 get: {err}")))?;
        let mut body = object.body.into_async_read();
        let mut file = fs::File::create(&path)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        tokio::io::copy(&mut body, &mut file)
            .await
            .map_err(|err| ApiError::internal(format!("s3 download: {err}")))?;
        file.flush()
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        Ok(path)
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

/// Shared helper so callers can hold an `Arc` without caring which backend is active.
pub type SharedBlobStore = Arc<BlobStore>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fs_put_bytes_round_trip() {
        let dir = std::env::temp_dir().join(format!("genoma-blob-{}", Uuid::new_v4()));
        let store = BlobStore::Fs(FsBlobStore::new(dir.clone()).await.unwrap());
        store.put_bytes("a/b.txt", b"hello").await.unwrap();
        let path = store.local_path_or_download("a/b.txt").await.unwrap();
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(bytes, b"hello");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    #[ignore = "requires MinIO from docker compose"]
    async fn s3_put_get_against_minio() {
        let endpoint = std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:9000".into());
        let dir = std::env::temp_dir().join(format!("genoma-s3-{}", Uuid::new_v4()));
        let store = BlobStore::S3(
            S3BlobStore::new(
                &endpoint,
                "us-east-1",
                "genoma",
                "genoma",
                "genoma-secret",
                true,
                dir.clone(),
            )
            .await
            .unwrap(),
        );
        let key = format!("test/{}/blob.bin", Uuid::new_v4());
        store.put_bytes(&key, b"minio-ok").await.unwrap();
        let path = store.local_path_or_download(&key).await.unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"minio-ok");
        let _ = std::fs::remove_dir_all(dir);
    }
}
