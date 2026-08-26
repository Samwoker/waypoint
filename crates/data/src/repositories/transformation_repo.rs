use std::sync::Arc;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use relay_core::error::CoreError;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transformation {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub subscription_id: Uuid,
    pub rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct TransformationRepository {
    pool: Arc<PgPool>,
}

impl TransformationRepository {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list_by_subscription(
        &self,
        tenant_id: Uuid,
        subscription_id: Uuid,
    ) -> Result<Vec<Transformation>, CoreError> {
        let rows = sqlx::query_as::<_, Transformation>(
            r#"
            SELECT t.id, t.tenant_id, t.subscription_id, t.rules, t.created_at, t.updated_at
            FROM transformations t
            JOIN subscriptions s ON s.id = t.subscription_id
            WHERE t.tenant_id = $1
              AND t.subscription_id = $2
              AND s.tenant_id = $1
            ORDER BY t.created_at ASC
            "#,
        )
        .bind(tenant_id)
        .bind(subscription_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing transformations: {e}")))?;

        Ok(rows)
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        subscription_id: Uuid,
        rules: serde_json::Value,
    ) -> Result<Transformation, CoreError> {
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, Transformation>(
            r#"
            INSERT INTO transformations (id, tenant_id, subscription_id, rules)
            VALUES ($1, $2, $3, $4)
            RETURNING id, tenant_id, subscription_id, rules, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(subscription_id)
        .bind(&rules)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error creating transformation: {e}")))?;

        Ok(row)
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        transformation_id: Uuid,
        rules: serde_json::Value,
    ) -> Result<Transformation, CoreError> {
        let row = sqlx::query_as::<_, Transformation>(
            r#"
            UPDATE transformations
            SET rules = $1, updated_at = NOW()
            WHERE id = $2
              AND tenant_id = $3
            RETURNING id, tenant_id, subscription_id, rules, created_at, updated_at
            "#,
        )
        .bind(&rules)
        .bind(transformation_id)
        .bind(tenant_id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating transformation: {e}")))?
        .ok_or_else(|| CoreError::NotFound(format!("Transformation '{transformation_id}' not found")))?;

        Ok(row)
    }

    pub async fn delete(
        &self,
        tenant_id: Uuid,
        transformation_id: Uuid,
    ) -> Result<(), CoreError> {
        let result = sqlx::query(
            "DELETE FROM transformations WHERE id = $1 AND tenant_id = $2"
        )
        .bind(transformation_id)
        .bind(tenant_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error deleting transformation: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(format!("Transformation '{transformation_id}' not found")));
        }

        Ok(())
    }

    pub async fn find_by_tenant_and_id(
        &self,
        tenant_id: Uuid,
        transformation_id: Uuid,
    ) -> Result<Option<Transformation>, CoreError> {
        let row = sqlx::query_as::<_, Transformation>(
            r#"
            SELECT id, tenant_id, subscription_id, rules, created_at, updated_at
            FROM transformations
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(transformation_id)
        .bind(tenant_id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error finding transformation: {e}")))?;

        Ok(row)
    }
}
