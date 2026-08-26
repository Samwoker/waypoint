use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::User;

#[derive(Clone, Debug)]
pub struct UserRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, CoreError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, tenant_id, email, password_hash, role::text AS role, created_at, updated_at
            FROM users
            WHERE LOWER(email) = LOWER($1) AND status = 'active'
            LIMIT 1
            "#,
        )
        .bind(email)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching user by email: {e}")))?;

        Ok(user)
    }

    pub async fn find_by_tenant_and_email(&self, tenant_id: Uuid, email: &str) -> Result<Option<User>, CoreError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, tenant_id, email, password_hash, role::text AS role, created_at, updated_at
            FROM users
            WHERE tenant_id = $1 AND LOWER(email) = LOWER($2) AND status = 'active'
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(email)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching user: {e}")))?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, CoreError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, tenant_id, email, password_hash, role::text AS role, created_at, updated_at
            FROM users
            WHERE id = $1 AND status = 'active'
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching user by id: {e}")))?;

        Ok(user)
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        email: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<User, CoreError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (tenant_id, email, password_hash, role, status)
            VALUES ($1, $2, $3, $4::user_role, 'active')
            RETURNING id, tenant_id, email, password_hash, role::text AS role, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error creating user: {e}")))?;

        Ok(user)
    }

    pub async fn update_last_login(&self, id: Uuid) -> Result<(), CoreError> {
        sqlx::query(
            r#"
            UPDATE users
            SET last_login_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating last login: {e}")))?;

        Ok(())
    }
}
