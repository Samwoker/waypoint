use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;
use tracing::info;

mod delivery_poller;
mod fanout_consumer;

use relay_core::config::Config;
use data::pool::create_pg_pool;
use data::queue::RedisQueue;
use domain::services::{DeliveryService, FanoutService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load()?;
    info!("Starting Webhook Relay Worker in {} environment", config.environment);

    let pool = create_pg_pool(&config.database_url).await?;
    let pool = Arc::new(pool);
    info!("Connected to Postgres");

    let queue = RedisQueue::new(&config.redis_url).await?;
    let queue = Arc::new(Mutex::new(queue));
    info!("Connected to Redis");

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let fanout_service = Arc::new(FanoutService::new(pool.clone(), queue.clone()));
    let delivery_service = Arc::new(DeliveryService::new(pool.clone(), http_client));

    let fanout_handle = tokio::spawn(fanout_consumer::run(fanout_service, queue));
    let delivery_handle = tokio::spawn(delivery_poller::run(delivery_service));

    tokio::select! {
        res = fanout_handle => {
            info!("Fanout consumer task finished: {:?}", res);
        }
        res = delivery_handle => {
            info!("Delivery poller task finished: {:?}", res);
        }
        _ = signal::ctrl_c() => {
            info!("Shutdown signal received, exiting gracefully");
        }
    }

    Ok(())
}
