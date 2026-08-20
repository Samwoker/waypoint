use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use data::queue::RedisQueue;

#[derive(Clone)]
pub struct FanoutService {
    pub pool: Arc<PgPool>,
    pub queue: Arc<tokio::sync::Mutex<RedisQueue>>,
}

impl FanoutService {
    pub fn new(pool: Arc<PgPool>, queue: Arc<tokio::sync::Mutex<RedisQueue>>) -> Self {
        Self { pool, queue }
    }

    pub async fn fan_out_event(&self, _event_id: Uuid) -> Result<usize, CoreError> {
        todo!()
    }
}
