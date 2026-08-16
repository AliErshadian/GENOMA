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

pub async fn connect_redis(config: &AppConfig) -> Option<redis::aio::MultiplexedConnection> {
    let url = config.redis_url.as_ref()?;
    match redis::Client::open(url.as_str()) {
        Ok(client) => match client.get_multiplexed_async_connection().await {
            Ok(connection) => {
                info!("connected to Redis");
                Some(connection)
            }
            Err(err) => {
                warn!(error = %err, "Redis unavailable; progress stays in-process");
                None
            }
        },
        Err(err) => {
            warn!(error = %err, "invalid REDIS_URL; progress stays in-process");
            None
        }
    }
}
