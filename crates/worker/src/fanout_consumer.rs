use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::info;

use data::queue::RedisQueue;
use domain::services::FanoutService;

pub async fn run(
    fanout_service: Arc<FanoutService>,
    _queue: Arc<Mutex<RedisQueue>>,
) -> anyhow::Result<()> {
    info!("Starting Fanout Consumer stream loop");

    loop {
        // In full implementation:
        // 1. Read events from Redis stream using XREADGROUP
        // 2. For each event, invoke fanout_service.fan_out_event(event_id)
        // 3. ACK the message with XACK
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _ = &fanout_service;
    }
}
