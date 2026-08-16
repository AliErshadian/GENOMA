use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::config::AppConfig;

pub async fn connect_postgres(config: &AppConfig) -> Option<PgPool> {
    let url = config.database_url.as_ref()?;
    match PgPoolOptions::new().max_connections(5).connect(url).await {
        Ok(pool) => {
            if let Err(err) = sqlx::migrate!("./migrations").run(&pool).await {
                warn!(error = %err, "failed to run postgres migrations; continuing with in-memory store");
                return None;
            }
            info!("connected to PostgreSQL");
            Some(pool)
        }
        Err(err) => {
            warn!(error = %err, "PostgreSQL unavailable; using in-memory analysis store");
            None
        }
    }
}
