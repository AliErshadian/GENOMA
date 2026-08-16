use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("π source error: {0}")]
    Pi(String),

    #[error("analysis failed: {0}")]
    Analysis(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("upload rejected: {0}")]
    Upload(String),

    #[error("job error: {0}")]
    Job(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn pi(msg: impl Into<String>) -> Self {
        Self::Pi(msg.into())
    }

    pub fn analysis(msg: impl Into<String>) -> Self {
        Self::Analysis(msg.into())
    }

    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }

    pub fn upload(msg: impl Into<String>) -> Self {
        Self::Upload(msg.into())
    }
}

impl From<PathBuf> for Error {
    fn from(value: PathBuf) -> Self {
        Self::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("path not found: {}", value.display()),
        ))
    }
}
