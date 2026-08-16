use std::path::PathBuf;
use std::time::Duration;

use genoma_core::{AnalysisConfig, AnalysisLevel, ChunkSize};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub public_url: String,
    pub database_url: Option<String>,
    pub redis_url: Option<String>,
    pub blob_dir: PathBuf,
    pub pi_digits_path: PathBuf,
    pub demo_dir: PathBuf,
    pub git_repos_dir: PathBuf,
    pub max_upload_bytes: u64,
    pub cors_origin: String,
    pub default_analysis: AnalysisConfig,
    pub rate_limit_per_minute: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let chunk_size = std::env::var("GENOMA_DEFAULT_CHUNK_SIZE")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .and_then(|bytes| ChunkSize::from_bytes(bytes).ok())
            .unwrap_or(ChunkSize::Mb1);

        Self {
            host: env_or("GENOMA_HOST", "127.0.0.1"),
            port: env_or("GENOMA_PORT", "8080").parse().unwrap_or(8080),
            public_url: env_or("GENOMA_PUBLIC_URL", "http://localhost:8080"),
            database_url: nonempty("DATABASE_URL"),
            redis_url: nonempty("REDIS_URL"),
            blob_dir: PathBuf::from(env_or("GENOMA_BLOB_DIR", "./data/uploads")),
            pi_digits_path: PathBuf::from(env_or(
                "GENOMA_PI_DIGITS_PATH",
                "./data/pi/pi-digits.bin",
            )),
            demo_dir: PathBuf::from(env_or("GENOMA_DEMO_DIR", "./data/demos")),
            git_repos_dir: PathBuf::from(env_or("GENOMA_GIT_REPOS_DIR", "./data/repos")),
            max_upload_bytes: env_or("GENOMA_MAX_UPLOAD_BYTES", "2147483648")
                .parse()
                .unwrap_or(2 * 1024 * 1024 * 1024),
            cors_origin: env_or("CORS_ORIGIN", "http://localhost:3000"),
            default_analysis: AnalysisConfig {
                chunk_size,
                level: AnalysisLevel::Balanced,
                pi_base_offset: 0,
                generator_version: genoma_core::GENERATOR_VERSION.to_string(),
            },
            rate_limit_per_minute: env_or("GENOMA_RATE_LIMIT_PER_MINUTE", "30")
                .parse()
                .unwrap_or(30),
        }
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn request_timeout() -> Duration {
        Duration::from_secs(60 * 30)
    }

    pub fn for_tests(root: impl Into<std::path::PathBuf>) -> Self {
        let root = root.into();
        Self {
            host: "127.0.0.1".into(),
            port: 0,
            public_url: "http://localhost:8080".into(),
            database_url: None,
            redis_url: None,
            blob_dir: root.join("uploads"),
            pi_digits_path: root.join("data/pi/pi-digits.bin"),
            demo_dir: root.join("data/demos"),
            git_repos_dir: root.join("data/repos"),
            max_upload_bytes: 32 * 1024 * 1024,
            cors_origin: "http://localhost:3000".into(),
            default_analysis: AnalysisConfig::default().with_chunk_size(ChunkSize::Kb4),
            rate_limit_per_minute: 120,
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}
