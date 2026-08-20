use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use domain::services::DeliveryService;

pub async fn run(delivery_service: Arc<DeliveryService>) -> anyhow::Result<()> {
    info!("Starting Delivery Poller scheduler loop");

    loop {
        // In full implementation:
        // 1. Query delivery_repo.list_due_deliveries(batch_size)
        // 2. Spawn concurrent delivery tasks via delivery_service.attempt_delivery(id)
        // 3. Sleep interval or poll delay
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _ = &delivery_service;
    }
}
