use std::io::Cursor;
use std::path::PathBuf;

use analysis_engine::{analyze_reader, NoopProgress};
use genoma_api::{app, build_state, config::AppConfig};
use genoma_core::AnalysisConfig;
use pi_engine::FilePiSource;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let mut args = std::env::args().skip(1);
    if let Some(cmd) = args.next() {
        if cmd == "emit-demo-dna" {
            return emit_demo_dna(args.next());
        }
    }

    let config = AppConfig::from_env();
    let bind = config.bind_addr();
    let state = build_state(config)
        .await
        .map_err(|err| anyhow::anyhow!(err.message))?;
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("GENOMA API listening on {bind}");
    axum::serve(listener, app(state)).await?;
    Ok(())
}

fn emit_demo_dna(out: Option<String>) -> anyhow::Result<()> {
    let config = AppConfig::from_env();
    let source = FilePiSource::load(&config.pi_digits_path)?;
    let demo = config.demo_dir.join("sample.txt");
    let bytes = std::fs::read(&demo)?;
    let result = analyze_reader(
        Cursor::new(bytes),
        &source,
        &AnalysisConfig::default(),
        None,
        &NoopProgress,
    )?;
    let json = serde_json::to_string_pretty(&result.dna)?;
    let path = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("apps/web/public/demo-dna.json"));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, json)?;
    println!("wrote {}", path.display());
    Ok(())
}
