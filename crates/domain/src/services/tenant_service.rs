use std::sync::Arc;
use data::repositories::TenantRepository;
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::dto::{CreateTenantInput, DailyEventCount, TenantUsageView, TenantView, UpdateTenantInput};

#[derive(Clone)]
pub struct TenantService {
    pub pool: Arc<PgPool>,
}

impl TenantService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_tenant(&self, input: CreateTenantInput) -> Result<TenantView, CoreError> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(CoreError::Validation("Tenant name cannot be empty".to_string()));
        }

        let slug = input.slug.trim();
        if slug.is_empty() {
            return Err(CoreError::Validation("Tenant slug cannot be empty".to_string()));
        }

        let repo = TenantRepository::new(&self.pool);
        let tenant = repo.create(name, slug).await?;

        Ok(TenantView {
            id: tenant.id,
            name: tenant.name,
            slug: tenant.slug,
            created_at: tenant.created_at,
            updated_at: tenant.updated_at,
        })
    }

    pub async fn get_tenant(&self, id: Uuid) -> Result<Option<TenantView>, CoreError> {
        let repo = TenantRepository::new(&self.pool);
        let tenant = repo.find_by_id(id).await?;

        Ok(tenant.map(|t| TenantView {
            id: t.id,
            name: t.name,
            slug: t.slug,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }))
    }

    pub async fn update_tenant(&self, id: Uuid, input: UpdateTenantInput) -> Result<TenantView, CoreError> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(CoreError::Validation("Tenant name cannot be empty".to_string()));
        }

        let repo = TenantRepository::new(&self.pool);
        let tenant = repo.update(id, name).await?;

        Ok(TenantView {
            id: tenant.id,
            name: tenant.name,
            slug: tenant.slug,
            created_at: tenant.created_at,
            updated_at: tenant.updated_at,
        })
    }

    pub async fn list_tenants(&self, limit: i64, offset: i64) -> Result<Vec<TenantView>, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };
        let offset = if offset < 0 { 0 } else { offset };

        let repo = TenantRepository::new(&self.pool);
        let tenants = repo.list(limit, offset).await?;

        Ok(tenants
            .into_iter()
            .map(|t| TenantView {
                id: t.id,
                name: t.name,
                slug: t.slug,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect())
    }

    pub async fn get_tenant_usage(
        &self,
        caller_tenant_id: Uuid,
        is_admin: bool,
        target_tenant_id: Uuid,
        period: Option<&str>,
    ) -> Result<TenantUsageView, CoreError> {
        // Authorization check: normal tenant users may only access their own tenant
        if caller_tenant_id != target_tenant_id && !is_admin {
            return Err(CoreError::Forbidden("Access denied to tenant usage".to_string()));
        }

        // Validate target tenant exists
        let repo = TenantRepository::new(&self.pool);
        if repo.find_by_id(target_tenant_id).await?.is_none() {
            return Err(CoreError::NotFound(format!("Tenant '{target_tenant_id}' not found")));
        }

        // Parse period parameter
        let period_str = match period.map(|p| p.trim()).filter(|p| !p.is_empty()) {
            Some(p) => p,
            None => "30d",
        };

        let interval_sql = match period_str {
            "24h" | "1d" | "1 day" => "1 day",
            "7d" | "7 days" => "7 days",
            "30d" | "30 days" => "30 days",
            "60d" | "60 days" => "60 days",
            "90d" | "90 days" => "90 days",
            "365d" | "1y" | "1 year" => "365 days",
            other => {
                if let Some(days_str) = other.strip_suffix('d') {
                    if let Ok(days) = days_str.parse::<u32>() {
                        if days > 0 && days <= 3650 {
                            // valid
                        } else {
                            return Err(CoreError::Validation("Period must be between 1 and 3650 days".to_string()));
                        }
                    } else {
                        return Err(CoreError::Validation(format!("Invalid period format: '{other}'")));
                    }
                } else {
                    return Err(CoreError::Validation(format!("Invalid period format: '{other}'. Examples: 24h, 7d, 30d, 90d")));
                }
                other
            }
        };

        let (total_events, total_delivery_attempts, daily_counts) = repo
            .get_usage(target_tenant_id, interval_sql)
            .await?;

        let daily_events = daily_counts
            .into_iter()
            .map(|(date, count)| DailyEventCount { date, count })
            .collect();

        Ok(TenantUsageView {
            tenant_id: target_tenant_id,
            period: period_str.to_string(),
            total_events,
            total_delivery_attempts,
            daily_events,
        })
    }
}
