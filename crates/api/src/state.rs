use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::Mutex;

use relay_core::config::Config;
use data::queue::RedisQueue;
use domain::services::{
    AuditService, AuthService, DeliveryService, DestinationService, FanoutService, IngestionService,
    SourceService, SubscriptionService, TenantService,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: Arc<PgPool>,
    pub queue: Arc<Mutex<RedisQueue>>,
    pub auth_service: Arc<AuthService>,
    pub tenant_service: Arc<TenantService>,
    pub source_service: Arc<SourceService>,
    pub destination_service: Arc<DestinationService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub ingestion_service: Arc<IngestionService>,
    pub fanout_service: Arc<FanoutService>,
    pub delivery_service: Arc<DeliveryService>,
    pub audit_service: Arc<AuditService>,
}

impl AppState {
    pub fn new(
        config: Config,
        pool: PgPool,
        queue: RedisQueue,
    ) -> Result<Self, relay_core::error::CoreError> {
        let pool = Arc::new(pool);
        let queue = Arc::new(Mutex::new(queue));

        let mut key_bytes = [0u8; 32];
        let decoded = hex::decode(&config.data_encryption_key)
            .map_err(|e| relay_core::error::CoreError::Crypto(format!("Invalid hex encryption key: {e}")))?;
        if decoded.len() != 32 {
            return Err(relay_core::error::CoreError::Crypto("Encryption key must be 32 bytes".to_string()));
        }
        key_bytes.copy_from_slice(&decoded);

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| relay_core::error::CoreError::Internal(format!("Failed to build HTTP client: {e}")))?;

        let auth_service = Arc::new(AuthService::new(pool.clone(), config.jwt_secret.clone()));
        let tenant_service = Arc::new(TenantService::new(pool.clone()));
        let source_service = Arc::new(SourceService::new(pool.clone(), key_bytes));
        let destination_service = Arc::new(DestinationService::new(pool.clone(), key_bytes));
        let subscription_service = Arc::new(SubscriptionService::new(pool.clone()));
        let ingestion_service = Arc::new(IngestionService::new(pool.clone(), queue.clone()));
        let fanout_service = Arc::new(FanoutService::new(pool.clone(), queue.clone()));
        let delivery_service = Arc::new(DeliveryService::new(pool.clone(), http_client));
        let audit_service = Arc::new(AuditService::new(pool.clone()));

        Ok(Self {
            config,
            pool,
            queue,
            auth_service,
            tenant_service,
            source_service,
            destination_service,
            subscription_service,
            ingestion_service,
            fanout_service,
            delivery_service,
            audit_service,
        })
    }
}
