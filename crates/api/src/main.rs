use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

use relay_core::config::Config;
use data::pool::{create_pg_pool, run_migrations};
use data::queue::RedisQueue;
use api::{create_router, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load()?;
    info!("Starting Webhook Relay API in {} environment", config.environment);

    let pool = create_pg_pool(&config.database_url).await?;
    info!("Connected to Postgres");

    run_migrations(&pool).await?;
    info!("Database migrations executed successfully");

    let queue = RedisQueue::new(&config.redis_url).await?;
    info!("Connected to Redis");

    let app_state = AppState::new(config.clone(), pool, queue)?;
    let app = create_router(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.api_port));
    info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
