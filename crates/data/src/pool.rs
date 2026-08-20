use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

use relay_core::error::CoreError;

pub async fn create_pg_pool(database_url: &str) -> Result<PgPool, CoreError> {
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(|e| CoreError::Internal(format!("Failed to connect to Postgres: {e}")))
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), CoreError> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database migration failed: {e}")))?;
    Ok(())
}
