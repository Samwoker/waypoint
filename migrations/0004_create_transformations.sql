-- ============================================================
-- TRANSFORMATIONS
-- Stores field-mapping rules associated with a subscription.
-- When present, the worker transforms the event payload before
-- delivery using these rules.
-- ============================================================

CREATE TABLE IF NOT EXISTS transformations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    subscription_id UUID NOT NULL
        REFERENCES subscriptions(id)
        ON DELETE CASCADE,

    -- JSON array of { "source_path": "$.x", "dest_path": "$.y" }
    rules JSONB NOT NULL DEFAULT '[]'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT transformations_rules_is_array
        CHECK (jsonb_typeof(rules) = 'array')
);

CREATE INDEX IF NOT EXISTS idx_transformations_subscription_id
    ON transformations(subscription_id);

CREATE INDEX IF NOT EXISTS idx_transformations_tenant_id
    ON transformations(tenant_id);
