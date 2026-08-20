-- ============================================================
-- DELIVERIES QUERY INDEX
-- Adds optimized composite index for delivery query filtering and pagination.
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_deliveries_query
    ON deliveries(tenant_id, destination_id, status, created_at DESC);
