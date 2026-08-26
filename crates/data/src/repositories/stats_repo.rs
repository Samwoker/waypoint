use std::sync::Arc;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use relay_core::error::CoreError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OverviewStats {
    pub total_events: i64,
    pub total_deliveries: i64,
    pub delivered_count: i64,
    pub success_rate: f64,
    pub p50_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyEventVolume {
    pub day: String,
    pub event_count: i64,
    pub signature_valid_count: i64,
    pub signature_invalid_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DestinationStats {
    pub destination_id: Uuid,
    pub destination_name: String,
    pub destination_url: String,
    pub status: String,
    pub consecutive_failures: Option<i32>,
    pub circuit_opened_at: Option<DateTime<Utc>>,
    pub total_deliveries: i64,
    pub delivered_count: i64,
    pub failed_count: i64,
    pub success_rate: f64,
    pub p50_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyDeliveryVolume {
    pub day: String,
    pub total: i64,
    pub delivered: i64,
    pub failed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TimeseriesBucket {
    pub bucket: DateTime<Utc>,
    pub value: f64,
}

pub struct StatsRepository {
    pool: Arc<PgPool>,
}

impl StatsRepository {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }

    /// Parse period string like "24h", "7d", "30d" into an interval string for Postgres.
    pub fn parse_period_interval(period: &str) -> Result<&'static str, CoreError> {
        match period {
            "1h"  => Ok("1 hour"),
            "6h"  => Ok("6 hours"),
            "12h" => Ok("12 hours"),
            "24h" => Ok("24 hours"),
            "7d"  => Ok("7 days"),
            "30d" => Ok("30 days"),
            "90d" => Ok("90 days"),
            _ => Err(CoreError::Validation(format!(
                "Invalid period '{period}'. Valid values: 1h, 6h, 12h, 24h, 7d, 30d, 90d"
            ))),
        }
    }

    pub async fn get_overview_stats(
        &self,
        tenant_id: Uuid,
        interval: &str,
    ) -> Result<OverviewStats, CoreError> {
        // Use a safe interval string we control (not user-provided directly into SQL)
        let row = sqlx::query_as::<_, OverviewStats>(
            r#"
            SELECT
                COALESCE((
                    SELECT COUNT(*)
                    FROM events
                    WHERE tenant_id = $1
                      AND received_at >= NOW() - $2::interval
                ), 0)::bigint AS total_events,

                COALESCE((
                    SELECT COUNT(*)
                    FROM deliveries
                    WHERE tenant_id = $1
                      AND created_at >= NOW() - $2::interval
                ), 0)::bigint AS total_deliveries,

                COALESCE((
                    SELECT COUNT(*)
                    FROM deliveries
                    WHERE tenant_id = $1
                      AND status = 'delivered'
                      AND created_at >= NOW() - $2::interval
                ), 0)::bigint AS delivered_count,

                CASE
                    WHEN COALESCE((
                        SELECT COUNT(*)
                        FROM deliveries
                        WHERE tenant_id = $1
                          AND created_at >= NOW() - $2::interval
                    ), 0) = 0 THEN 0.0
                    ELSE ROUND(
                        COALESCE((
                            SELECT COUNT(*)::numeric
                            FROM deliveries
                            WHERE tenant_id = $1
                              AND status = 'delivered'
                              AND created_at >= NOW() - $2::interval
                        ), 0)
                        /
                        COALESCE((
                            SELECT COUNT(*)::numeric
                            FROM deliveries
                            WHERE tenant_id = $1
                              AND created_at >= NOW() - $2::interval
                        ), 1)
                        * 100, 2
                    )
                END::double precision AS success_rate,

                (
                    SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY da.duration_ms)
                    FROM delivery_attempts da
                    JOIN deliveries d ON d.id = da.delivery_id
                    WHERE d.tenant_id = $1
                      AND da.created_at >= NOW() - $2::interval
                      AND da.duration_ms IS NOT NULL
                ) AS p50_latency_ms,

                (
                    SELECT percentile_cont(0.95) WITHIN GROUP (ORDER BY da.duration_ms)
                    FROM delivery_attempts da
                    JOIN deliveries d ON d.id = da.delivery_id
                    WHERE d.tenant_id = $1
                      AND da.created_at >= NOW() - $2::interval
                      AND da.duration_ms IS NOT NULL
                ) AS p95_latency_ms
            "#,
        )
        .bind(tenant_id)
        .bind(interval)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching overview stats: {e}")))?;

        Ok(row)
    }

    pub async fn get_source_daily_stats(
        &self,
        tenant_id: Uuid,
        source_id: Uuid,
        interval: &str,
    ) -> Result<Vec<DailyEventVolume>, CoreError> {
        // Verify source belongs to tenant first
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sources WHERE id = $1 AND tenant_id = $2)"
        )
        .bind(source_id)
        .bind(tenant_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error checking source: {e}")))?;

        if !exists {
            return Err(CoreError::NotFound(format!("Source '{source_id}' not found")));
        }

        let rows = sqlx::query_as::<_, DailyEventVolume>(
            r#"
            SELECT
                TO_CHAR(date_trunc('day', e.received_at), 'YYYY-MM-DD') AS day,
                COUNT(*)::bigint AS event_count,
                0::bigint AS signature_valid_count,
                0::bigint AS signature_invalid_count
            FROM events e
            WHERE e.tenant_id = $1
              AND e.source_id = $2
              AND e.received_at >= NOW() - $3::interval
            GROUP BY date_trunc('day', e.received_at)
            ORDER BY date_trunc('day', e.received_at) ASC
            "#,
        )
        .bind(tenant_id)
        .bind(source_id)
        .bind(interval)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching source stats: {e}")))?;

        Ok(rows)
    }

    pub async fn get_destination_stats(
        &self,
        tenant_id: Uuid,
        destination_id: Uuid,
        interval: &str,
    ) -> Result<(DestinationStats, Vec<DailyDeliveryVolume>), CoreError> {
        // Verify destination belongs to tenant
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM destinations WHERE id = $1 AND tenant_id = $2)"
        )
        .bind(destination_id)
        .bind(tenant_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error checking destination: {e}")))?;

        if !exists {
            return Err(CoreError::NotFound(format!("Destination '{destination_id}' not found")));
        }

        let stats = sqlx::query_as::<_, DestinationStats>(
            r#"
            SELECT
                dest.id AS destination_id,
                dest.name AS destination_name,
                dest.url AS destination_url,
                dest.status::text AS status,
                NULL::integer AS consecutive_failures,
                NULL::timestamptz AS circuit_opened_at,
                COALESCE(COUNT(d.id), 0)::bigint AS total_deliveries,
                COALESCE(SUM(CASE WHEN d.status = 'delivered' THEN 1 ELSE 0 END), 0)::bigint AS delivered_count,
                COALESCE(SUM(CASE WHEN d.status IN ('failed', 'dead_letter', 'dead_lettered') THEN 1 ELSE 0 END), 0)::bigint AS failed_count,
                CASE
                    WHEN COUNT(d.id) = 0 THEN 0.0
                    ELSE ROUND(SUM(CASE WHEN d.status = 'delivered' THEN 1 ELSE 0 END)::numeric / COUNT(d.id)::numeric * 100, 2)
                END::double precision AS success_rate,
                (
                    SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY da.duration_ms)
                    FROM delivery_attempts da
                    JOIN deliveries d2 ON d2.id = da.delivery_id
                    WHERE d2.destination_id = $2
                      AND d2.tenant_id = $1
                      AND da.created_at >= NOW() - $3::interval
                      AND da.duration_ms IS NOT NULL
                ) AS p50_latency_ms,
                (
                    SELECT percentile_cont(0.95) WITHIN GROUP (ORDER BY da.duration_ms)
                    FROM delivery_attempts da
                    JOIN deliveries d2 ON d2.id = da.delivery_id
                    WHERE d2.destination_id = $2
                      AND d2.tenant_id = $1
                      AND da.created_at >= NOW() - $3::interval
                      AND da.duration_ms IS NOT NULL
                ) AS p95_latency_ms
            FROM destinations dest
            LEFT JOIN deliveries d
                ON d.destination_id = dest.id
               AND d.tenant_id = $1
               AND d.created_at >= NOW() - $3::interval
            WHERE dest.id = $2
              AND dest.tenant_id = $1
            GROUP BY dest.id, dest.name, dest.url, dest.status
            "#,
        )
        .bind(tenant_id)
        .bind(destination_id)
        .bind(interval)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching destination stats: {e}")))?;

        let daily = sqlx::query_as::<_, DailyDeliveryVolume>(
            r#"
            SELECT
                TO_CHAR(date_trunc('day', d.created_at), 'YYYY-MM-DD') AS day,
                COUNT(*)::bigint AS total,
                SUM(CASE WHEN d.status = 'delivered' THEN 1 ELSE 0 END)::bigint AS delivered,
                SUM(CASE WHEN d.status IN ('failed', 'dead_letter', 'dead_lettered') THEN 1 ELSE 0 END)::bigint AS failed
            FROM deliveries d
            WHERE d.destination_id = $1
              AND d.tenant_id = $2
              AND d.created_at >= NOW() - $3::interval
            GROUP BY date_trunc('day', d.created_at)
            ORDER BY date_trunc('day', d.created_at) ASC
            "#,
        )
        .bind(destination_id)
        .bind(tenant_id)
        .bind(interval)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching destination daily stats: {e}")))?;

        Ok((stats, daily))
    }

    pub async fn get_timeseries(
        &self,
        tenant_id: Uuid,
        metric: &str,
        bucket_sql: &str,
        interval: &str,
    ) -> Result<Vec<TimeseriesBucket>, CoreError> {
        // bucket_sql and interval are pre-validated, NOT user-provided raw strings
        let rows = match metric {
            "volume" => {
                sqlx::query_as::<_, TimeseriesBucket>(
                    &format!(r#"
                    SELECT
                        date_trunc('{bucket_sql}', received_at) AS bucket,
                        COUNT(*)::double precision AS value
                    FROM events
                    WHERE tenant_id = $1
                      AND received_at >= NOW() - $2::interval
                    GROUP BY date_trunc('{bucket_sql}', received_at)
                    ORDER BY bucket ASC
                    "#),
                )
                .bind(tenant_id)
                .bind(interval)
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| CoreError::Internal(format!("Database error fetching timeseries volume: {e}")))?
            }
            "success_rate" => {
                sqlx::query_as::<_, TimeseriesBucket>(
                    &format!(r#"
                    SELECT
                        date_trunc('{bucket_sql}', d.created_at) AS bucket,
                        CASE
                            WHEN COUNT(*) = 0 THEN 0.0
                            ELSE ROUND(SUM(CASE WHEN d.status = 'delivered' THEN 1.0 ELSE 0.0 END) / COUNT(*)::numeric * 100, 2)
                        END::double precision AS value
                    FROM deliveries d
                    WHERE d.tenant_id = $1
                      AND d.created_at >= NOW() - $2::interval
                    GROUP BY date_trunc('{bucket_sql}', d.created_at)
                    ORDER BY bucket ASC
                    "#),
                )
                .bind(tenant_id)
                .bind(interval)
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| CoreError::Internal(format!("Database error fetching timeseries success_rate: {e}")))?
            }
            _ => {
                return Err(CoreError::Validation(format!("Unknown metric '{metric}'")));
            }
        };

        Ok(rows)
    }

    pub async fn get_system_stats(&self) -> Result<SystemStats, CoreError> {
        let total_tenants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE status != 'deleted'")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or(0);

        let active_tenants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE status = 'active'")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or(0);

        let total_sources: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sources WHERE status != 'deleted'")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or(0);

        let total_destinations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM destinations WHERE status != 'deleted'")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or(0);

        let total_subscriptions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subscriptions WHERE status != 'deleted'")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or(0);

        let total_events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or(0);

        let total_deliveries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM deliveries")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or(0);

        let successful_deliveries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM deliveries WHERE status = 'delivered'")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or(0);

        let failed_deliveries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM deliveries WHERE status = 'failed'")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or(0);

        let dead_letter_deliveries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM deliveries WHERE status IN ('dead_letter', 'dead_lettered')")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or(0);

        Ok(SystemStats {
            total_tenants,
            active_tenants,
            total_sources,
            total_destinations,
            total_subscriptions,
            total_events,
            total_deliveries,
            successful_deliveries,
            failed_deliveries,
            dead_letter_deliveries,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_tenants: i64,
    pub active_tenants: i64,
    pub total_sources: i64,
    pub total_destinations: i64,
    pub total_subscriptions: i64,
    pub total_events: i64,
    pub total_deliveries: i64,
    pub successful_deliveries: i64,
    pub failed_deliveries: i64,
    pub dead_letter_deliveries: i64,
}
