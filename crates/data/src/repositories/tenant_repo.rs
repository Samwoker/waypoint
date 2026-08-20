use sqlx::{PgPool, Row};
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::Tenant;

#[derive(Clone, Debug)]
pub struct TenantRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> TenantRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Tenant>, CoreError> {
        let row = sqlx::query_as::<_, Tenant>(
            r#"
            SELECT id, name, slug, created_at, updated_at
            FROM tenants
            WHERE id = $1 AND status != 'deleted'
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching tenant: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Tenant>, CoreError> {
        let row = sqlx::query_as::<_, Tenant>(
            r#"
            SELECT id, name, slug, created_at, updated_at
            FROM tenants
            WHERE slug = $1 AND status != 'deleted'
            "#,
        )
        .bind(slug)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching tenant by slug: {e}")))?;

        Ok(row)
    }

    pub async fn create(&self, name: &str, slug: &str) -> Result<Tenant, CoreError> {
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, Tenant>(
            r#"
            INSERT INTO tenants (id, name, slug, status, plan, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, 'active', 'free', '{}'::jsonb, NOW(), NOW())
            RETURNING id, name, slug, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return CoreError::Conflict(format!("Tenant with slug '{slug}' already exists"));
                }
            }
            CoreError::Internal(format!("Database error creating tenant: {e}"))
        })?;

        Ok(row)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Tenant>, CoreError> {
        let rows = sqlx::query_as::<_, Tenant>(
            r#"
            SELECT id, name, slug, created_at, updated_at
            FROM tenants
            WHERE status != 'deleted'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing tenants: {e}")))?;

        Ok(rows)
    }

    pub async fn update(&self, id: Uuid, name: &str) -> Result<Tenant, CoreError> {
        let row = sqlx::query_as::<_, Tenant>(
            r#"
            UPDATE tenants
            SET name = $1, updated_at = NOW()
            WHERE id = $2 AND status != 'deleted'
            RETURNING id, name, slug, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating tenant: {e}")))?;

        Ok(row)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query(
            r#"
            UPDATE tenants
            SET status = 'deleted', updated_at = NOW()
            WHERE id = $1 AND status != 'deleted'
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error deleting tenant: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(format!("Tenant '{id}' not found")));
        }

        Ok(())
    }

    pub async fn get_usage(
        &self,
        tenant_id: Uuid,
        interval_str: &str,
    ) -> Result<(i64, i64, Vec<(String, i64)>), CoreError> {
        // 1. Daily events aggregation
        let daily_rows = sqlx::query(
            r#"
            SELECT
                TO_CHAR(date_trunc('day', received_at), 'YYYY-MM-DD') AS day,
                COUNT(*)::BIGINT AS event_count
            FROM events
            WHERE tenant_id = $1
              AND received_at >= NOW() - $2::interval
            GROUP BY date_trunc('day', received_at)
            ORDER BY date_trunc('day', received_at) ASC
            "#,
        )
        .bind(tenant_id)
        .bind(interval_str)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error aggregating daily events: {e}")))?;

        let mut daily_events = Vec::new();
        for r in daily_rows {
            let day: String = r.try_get("day").map_err(|e| CoreError::Internal(e.to_string()))?;
            let count: i64 = r.try_get("event_count").map_err(|e| CoreError::Internal(e.to_string()))?;
            daily_events.push((day, count));
        }

        // 2. Total events in period
        let total_events: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM events
            WHERE tenant_id = $1
              AND received_at >= NOW() - $2::interval
            "#,
        )
        .bind(tenant_id)
        .bind(interval_str)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error counting total events: {e}")))?;

        // 3. Total delivery attempts in period
        let total_delivery_attempts: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(da.id)::BIGINT
            FROM delivery_attempts da
            JOIN deliveries d ON da.delivery_id = d.id
            WHERE d.tenant_id = $1
              AND da.created_at >= NOW() - $2::interval
            "#,
        )
        .bind(tenant_id)
        .bind(interval_str)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error counting delivery attempts: {e}")))?;

        Ok((total_events, total_delivery_attempts, daily_events))
    }
}
