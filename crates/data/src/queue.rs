use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use relay_core::error::CoreError;

#[derive(Clone)]
pub struct RedisQueue {
    #[allow(dead_code)]
    client: redis::Client,
    connection: ConnectionManager,
}

impl std::fmt::Debug for RedisQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisQueue").finish()
    }
}

impl RedisQueue {
    pub async fn new(redis_url: &str) -> Result<Self, CoreError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| CoreError::Internal(format!("Invalid Redis URL: {e}")))?;
        let connection = ConnectionManager::new(client.clone())
            .await
            .map_err(|e| CoreError::Internal(format!("Failed to connect to Redis: {e}")))?;

        Ok(Self { client, connection })
    }

    pub async fn ensure_consumer_group(
        &mut self,
        stream: &str,
        group: &str,
    ) -> Result<(), CoreError> {
        let mut conn = self.connection.clone();
        // XGROUP CREATE stream group $ MKSTREAM
        let res: redis::RedisResult<()> = conn
            .xgroup_create_mkstream(stream, group, "$")
            .await;

        match res {
            Ok(()) => Ok(()),
            Err(e) => {
                // Ignore BUSYGROUP error if group already exists
                if e.to_string().contains("BUSYGROUP") {
                    Ok(())
                } else {
                    Err(CoreError::Internal(format!("Redis XGROUP CREATE failed: {e}")))
                }
            }
        }
    }

    pub async fn push_event(
        &mut self,
        stream: &str,
        payload: &[(&str, &str)],
    ) -> Result<String, CoreError> {
        let mut conn = self.connection.clone();
        let message_id: String = conn
            .xadd(stream, "*", payload)
            .await
            .map_err(|e| CoreError::Internal(format!("Redis XADD failed: {e}")))?;
        Ok(message_id)
    }

    pub async fn read_events(
        &mut self,
        stream: &str,
        group: &str,
        consumer: &str,
        count: usize,
    ) -> Result<redis::streams::StreamReadReply, CoreError> {
        let mut conn = self.connection.clone();
        let opts = redis::streams::StreamReadOptions::default()
            .group(group, consumer)
            .count(count);

        let reply: redis::streams::StreamReadReply = conn
            .xread_options(&[stream], &[">"], &opts)
            .await
            .map_err(|e| CoreError::Internal(format!("Redis XREADGROUP failed: {e}")))?;

        Ok(reply)
    }

    pub async fn ack(
        &mut self,
        stream: &str,
        group: &str,
        ids: &[&str],
    ) -> Result<usize, CoreError> {
        let mut conn = self.connection.clone();
        let acked_count: usize = conn
            .xack(stream, group, ids)
            .await
            .map_err(|e| CoreError::Internal(format!("Redis XACK failed: {e}")))?;
        Ok(acked_count)
    }

    pub async fn ping(&mut self) -> Result<(), CoreError> {
        let mut conn = self.connection.clone();
        let _: () = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| CoreError::Internal(format!("Redis PING failed: {e}")))?;
        Ok(())
    }
}
