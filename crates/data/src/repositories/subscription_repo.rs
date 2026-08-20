use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::Subscription;

#[derive(Clone, Debug)]
pub struct SubscriptionRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> SubscriptionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscription>, CoreError> {
        let row = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT
                id,
                tenant_id,
                source_id,
                destination_id,
                event_types,
                filter AS filter_rules,
                NULL::TEXT AS transformation_template,
                (status = 'active') AS is_active,
                created_at,
                updated_at
            FROM subscriptions
            WHERE id = $1 AND status != 'deleted'
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching subscription by id: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_tenant_and_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Subscription>, CoreError> {
        let row = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT
                id,
                tenant_id,
                source_id,
                destination_id,
                event_types,
                filter AS filter_rules,
                NULL::TEXT AS transformation_template,
                (status = 'active') AS is_active,
                created_at,
                updated_at
            FROM subscriptions
            WHERE tenant_id = $1 AND id = $2 AND status != 'deleted'
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching subscription: {e}")))?;

        Ok(row)
    }

    pub async fn list_by_source(&self, source_id: Uuid) -> Result<Vec<Subscription>, CoreError> {
        let rows = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT
                id,
                tenant_id,
                source_id,
                destination_id,
                event_types,
                filter AS filter_rules,
                NULL::TEXT AS transformation_template,
                (status = 'active') AS is_active,
                created_at,
                updated_at
            FROM subscriptions
            WHERE source_id = $1 AND status = 'active'
            ORDER BY created_at ASC
            "#,
        )
        .bind(source_id)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing subscriptions by source: {e}")))?;

        Ok(rows)
    }

    pub async fn list_by_tenant(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Subscription>, CoreError> {
        let rows = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT
                id,
                tenant_id,
                source_id,
                destination_id,
                event_types,
                filter AS filter_rules,
                NULL::TEXT AS transformation_template,
                (status = 'active') AS is_active,
                created_at,
                updated_at
            FROM subscriptions
            WHERE tenant_id = $1 AND status != 'deleted'
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing subscriptions: {e}")))?;

        Ok(rows)
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        source_id: Uuid,
        destination_id: Uuid,
        event_types: Vec<String>,
        filter_rules: Option<serde_json::Value>,
        _transformation_template: Option<&str>,
    ) -> Result<Subscription, CoreError> {
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, Subscription>(
            r#"
            INSERT INTO subscriptions (
                id, tenant_id, source_id, destination_id, event_types, filter, status, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, 'active', NOW(), NOW()
            )
            RETURNING
                id,
                tenant_id,
                source_id,
                destination_id,
                event_types,
                filter AS filter_rules,
                NULL::TEXT AS transformation_template,
                (status = 'active') AS is_active,
                created_at,
                updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(source_id)
        .bind(destination_id)
        .bind(&event_types)
        .bind(filter_rules)
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23503") {
                    return CoreError::NotFound("Referenced source or destination does not exist".to_string());
                }
            }
            CoreError::Internal(format!("Database error creating subscription: {e}"))
        })?;

        Ok(row)
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        event_types: Option<Vec<String>>,
        filter_rules: Option<serde_json::Value>,
        _transformation_template: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Subscription, CoreError> {
        let existing = self
            .find_by_tenant_and_id(tenant_id, id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Subscription '{id}' not found")))?;

        let new_event_types = event_types.unwrap_or(existing.event_types);
        let new_filter = filter_rules.or(existing.filter_rules);
        let new_status = match is_active {
            Some(true) => "active",
            Some(false) => "inactive",
            None => {
                if existing.is_active {
                    "active"
                } else {
                    "inactive"
                }
            }
        };

        let row = sqlx::query_as::<_, Subscription>(
            r#"
            UPDATE subscriptions
            SET
                event_types = $1,
                filter = $2,
                status = $3::subscription_status,
                updated_at = NOW()
            WHERE tenant_id = $4 AND id = $5 AND status != 'deleted'
            RETURNING
                id,
                tenant_id,
                source_id,
                destination_id,
                event_types,
                filter AS filter_rules,
                NULL::TEXT AS transformation_template,
                (status = 'active') AS is_active,
                created_at,
                updated_at
            "#,
        )
        .bind(&new_event_types)
        .bind(new_filter)
        .bind(new_status)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating subscription: {e}")))?;

        Ok(row)
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query(
            r#"
            UPDATE subscriptions
            SET status = 'deleted', updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2 AND status != 'deleted'
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error deleting subscription: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(format!("Subscription '{id}' not found")));
        }

        Ok(())
    }
}
