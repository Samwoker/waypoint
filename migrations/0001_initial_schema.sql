-- ============================================================
-- RelayCore / Webhook Event Platform
-- PostgreSQL Database Schema
-- ============================================================

-- ============================================================
-- EXTENSIONS
-- ============================================================

CREATE EXTENSION IF NOT EXISTS pgcrypto;


-- ============================================================
-- ENUM TYPES
-- ============================================================

CREATE TYPE tenant_status AS ENUM (
    'active',
    'suspended',
    'deleted'
);

CREATE TYPE user_role AS ENUM (
    'owner',
    'admin',
    'member',
    'viewer'
);

CREATE TYPE user_status AS ENUM (
    'active',
    'invited',
    'suspended',
    'deleted'
);

CREATE TYPE api_key_status AS ENUM (
    'active',
    'revoked',
    'expired'
);

CREATE TYPE source_status AS ENUM (
    'active',
    'inactive',
    'deleted'
);

CREATE TYPE destination_status AS ENUM (
    'active',
    'inactive',
    'disabled',
    'deleted'
);

CREATE TYPE subscription_status AS ENUM (
    'active',
    'paused',
    'disabled',
    'deleted'
);

CREATE TYPE event_status AS ENUM (
    'received',
    'queued',
    'processing',
    'completed',
    'failed'
);

CREATE TYPE delivery_status AS ENUM (
    'pending',
    'processing',
    'delivered',
    'retrying',
    'failed',
    'cancelled',
    'dead_lettered'
);

CREATE TYPE attempt_status AS ENUM (
    'started',
    'success',
    'failed',
    'timeout'
);

CREATE TYPE api_key_type AS ENUM (
    'secret',
    'publishable'
);


-- ============================================================
-- TENANTS
-- ============================================================
-- A tenant is a customer/company using RelayCore.
--
-- Example:
--   Acme Inc.
--   My E-commerce Company
--   Payment Platform
-- ============================================================

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    name VARCHAR(255) NOT NULL,

    slug VARCHAR(100) NOT NULL UNIQUE,

    status tenant_status NOT NULL DEFAULT 'active',

    plan VARCHAR(50) NOT NULL DEFAULT 'free',

    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- USERS
-- ============================================================
-- Dashboard users belonging to a tenant.
-- ============================================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    email VARCHAR(320) NOT NULL,

    password_hash TEXT NOT NULL,

    first_name VARCHAR(100),

    last_name VARCHAR(100),

    role user_role NOT NULL DEFAULT 'member',

    status user_status NOT NULL DEFAULT 'active',

    email_verified_at TIMESTAMPTZ,

    last_login_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT users_tenant_email_unique
        UNIQUE (tenant_id, email)
);


-- ============================================================
-- SESSIONS
-- ============================================================
-- Login sessions for dashboard users.
-- ============================================================

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    user_id UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    token_hash TEXT NOT NULL UNIQUE,

    ip_address INET,

    user_agent TEXT,

    expires_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- API KEYS
-- ============================================================
-- API keys authenticate applications communicating with
-- RelayCore.
--
-- IMPORTANT:
-- Never store the raw API key.
-- Store only the hash.
-- ============================================================

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    created_by UUID
        REFERENCES users(id)
        ON DELETE SET NULL,

    name VARCHAR(255) NOT NULL,

    key_prefix VARCHAR(30) NOT NULL,

    key_hash TEXT NOT NULL UNIQUE,

    type api_key_type NOT NULL DEFAULT 'secret',

    status api_key_status NOT NULL DEFAULT 'active',

    expires_at TIMESTAMPTZ,

    last_used_at TIMESTAMPTZ,

    revoked_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- SOURCES
-- ============================================================
-- A source represents where events originate.
--
-- Examples:
--   Payment backend
--   Mobile application
--   GitHub
--   Stripe
--   Internal ERP
-- ============================================================

CREATE TABLE sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,

    slug VARCHAR(100) NOT NULL,

    description TEXT,

    source_type VARCHAR(100) NOT NULL DEFAULT 'generic',

    status source_status NOT NULL DEFAULT 'active',

    signing_secret_encrypted TEXT,

    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT sources_tenant_slug_unique
        UNIQUE (tenant_id, slug)
);


-- ============================================================
-- WEBHOOK ENDPOINTS
-- ============================================================
-- Optional inbound endpoints generated by RelayCore.
--
-- Example:
-- https://hooks.example.com/in/abc123
-- ============================================================

CREATE TABLE webhook_endpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    source_id UUID NOT NULL
        REFERENCES sources(id)
        ON DELETE CASCADE,

    endpoint_key VARCHAR(100) NOT NULL UNIQUE,

    secret_encrypted TEXT,

    status source_status NOT NULL DEFAULT 'active',

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- DESTINATIONS
-- ============================================================
-- A destination is where RelayCore delivers webhooks.
--
-- Example:
-- https://api.customer.com/webhooks
-- ============================================================

CREATE TABLE destinations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,

    url TEXT NOT NULL,

    description TEXT,

    status destination_status NOT NULL DEFAULT 'active',

    secret_encrypted TEXT,

    timeout_ms INTEGER NOT NULL DEFAULT 10000,

    max_retries INTEGER NOT NULL DEFAULT 10,

    headers JSONB NOT NULL DEFAULT '{}'::jsonb,

    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT destinations_timeout_positive
        CHECK (timeout_ms > 0),

    CONSTRAINT destinations_max_retries_valid
        CHECK (max_retries >= 0)
);


-- ============================================================
-- SUBSCRIPTIONS
-- ============================================================
-- Defines which events should be delivered to which
-- destinations.
--
-- Example:
--
-- Source:
--   Payment Service
--
-- Destination:
--   Analytics Service
--
-- Event types:
--   payment.completed
--   payment.failed
-- ============================================================

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    source_id UUID
        REFERENCES sources(id)
        ON DELETE CASCADE,

    destination_id UUID NOT NULL
        REFERENCES destinations(id)
        ON DELETE CASCADE,

    event_types TEXT[] NOT NULL DEFAULT '{}',

    filter JSONB,

    status subscription_status NOT NULL DEFAULT 'active',

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- EVENTS
-- ============================================================
-- The actual events received by RelayCore.
--
-- Example:
--
-- event_type:
--   payment.completed
--
-- payload:
--   {
--      "payment_id": "pay_123",
--      "amount": 500
--   }
-- ============================================================

CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    source_id UUID NOT NULL
        REFERENCES sources(id)
        ON DELETE RESTRICT,

    event_type VARCHAR(255) NOT NULL,

    external_id VARCHAR(255),

    idempotency_key VARCHAR(255),

    payload JSONB NOT NULL,

    headers JSONB NOT NULL DEFAULT '{}'::jsonb,

    status event_status NOT NULL DEFAULT 'received',

    occurred_at TIMESTAMPTZ,

    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- EVENT IDEMPOTENCY
-- ============================================================
-- Prevents the same tenant from accidentally submitting
-- the same event multiple times using the same idempotency key.
-- ============================================================

CREATE UNIQUE INDEX idx_events_idempotency
ON events (
    tenant_id,
    idempotency_key
)
WHERE idempotency_key IS NOT NULL;


-- ============================================================
-- DELIVERIES
-- ============================================================
-- Represents one event being delivered to one destination.
--
-- One event can have many deliveries.
--
-- Event
--   ├── Destination A
--   ├── Destination B
--   └── Destination C
-- ============================================================

CREATE TABLE deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    event_id UUID NOT NULL
        REFERENCES events(id)
        ON DELETE CASCADE,

    subscription_id UUID
        REFERENCES subscriptions(id)
        ON DELETE SET NULL,

    destination_id UUID NOT NULL
        REFERENCES destinations(id)
        ON DELETE RESTRICT,

    status delivery_status NOT NULL DEFAULT 'pending',

    attempt_count INTEGER NOT NULL DEFAULT 0,

    max_attempts INTEGER NOT NULL DEFAULT 10,

    next_attempt_at TIMESTAMPTZ,

    delivered_at TIMESTAMPTZ,

    failed_at TIMESTAMPTZ,

    last_error TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT deliveries_attempt_count_valid
        CHECK (attempt_count >= 0),

    CONSTRAINT deliveries_max_attempts_valid
        CHECK (max_attempts >= 0),

    CONSTRAINT deliveries_unique_event_destination
        UNIQUE (event_id, destination_id)
);


-- ============================================================
-- DELIVERY ATTEMPTS
-- ============================================================
-- Stores every individual HTTP delivery attempt.
--
-- Example:
--
-- Attempt 1 -> HTTP 500
-- Attempt 2 -> HTTP 503
-- Attempt 3 -> HTTP 200
-- ============================================================

CREATE TABLE delivery_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    delivery_id UUID NOT NULL
        REFERENCES deliveries(id)
        ON DELETE CASCADE,

    attempt_number INTEGER NOT NULL,

    status attempt_status NOT NULL,

    request_url TEXT,

    request_headers JSONB,

    request_body JSONB,

    response_status INTEGER,

    response_headers JSONB,

    response_body TEXT,

    error_type VARCHAR(100),

    error_message TEXT,

    duration_ms INTEGER,

    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    completed_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT delivery_attempt_number_positive
        CHECK (attempt_number > 0),

    CONSTRAINT delivery_attempt_duration_valid
        CHECK (
            duration_ms IS NULL
            OR duration_ms >= 0
        ),

    CONSTRAINT delivery_attempt_unique_number
        UNIQUE (delivery_id, attempt_number)
);


-- ============================================================
-- DEAD LETTER QUEUE
-- ============================================================
-- Stores deliveries that failed after all retry attempts.
-- ============================================================

CREATE TABLE dead_letter_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    delivery_id UUID NOT NULL
        REFERENCES deliveries(id)
        ON DELETE CASCADE,

    reason TEXT NOT NULL,

    retry_count INTEGER NOT NULL,

    last_error TEXT,

    moved_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    resolved_at TIMESTAMPTZ,

    resolution VARCHAR(50),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT dlq_retry_count_valid
        CHECK (retry_count >= 0)
);


-- ============================================================
-- AUDIT LOGS
-- ============================================================
-- Records important configuration/security actions.
--
-- Examples:
--   user.created
--   destination.created
--   api_key.revoked
--   subscription.updated
-- ============================================================

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    user_id UUID
        REFERENCES users(id)
        ON DELETE SET NULL,

    action VARCHAR(100) NOT NULL,

    resource_type VARCHAR(100),

    resource_id UUID,

    ip_address INET,

    user_agent TEXT,

    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- USAGE RECORDS
-- ============================================================
-- Aggregated usage information for SaaS plans/billing.
-- ============================================================

CREATE TABLE usage_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    tenant_id UUID NOT NULL
        REFERENCES tenants(id)
        ON DELETE CASCADE,

    period_start DATE NOT NULL,

    period_end DATE NOT NULL,

    events_received BIGINT NOT NULL DEFAULT 0,

    deliveries_attempted BIGINT NOT NULL DEFAULT 0,

    deliveries_succeeded BIGINT NOT NULL DEFAULT 0,

    deliveries_failed BIGINT NOT NULL DEFAULT 0,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT usage_period_valid
        CHECK (period_end >= period_start),

    CONSTRAINT usage_events_received_valid
        CHECK (events_received >= 0),

    CONSTRAINT usage_deliveries_attempted_valid
        CHECK (deliveries_attempted >= 0),

    CONSTRAINT usage_deliveries_succeeded_valid
        CHECK (deliveries_succeeded >= 0),

    CONSTRAINT usage_deliveries_failed_valid
        CHECK (deliveries_failed >= 0),

    CONSTRAINT usage_tenant_period_unique
        UNIQUE (tenant_id, period_start, period_end)
);


-- ============================================================
-- INDEXES
-- ============================================================


-- USERS
CREATE INDEX idx_users_tenant_id
    ON users(tenant_id);

CREATE INDEX idx_users_email
    ON users(email);


-- SESSIONS
CREATE INDEX idx_sessions_user_id
    ON sessions(user_id);

CREATE INDEX idx_sessions_expires_at
    ON sessions(expires_at);


-- API KEYS
CREATE INDEX idx_api_keys_tenant_id
    ON api_keys(tenant_id);

CREATE INDEX idx_api_keys_status
    ON api_keys(status);

CREATE INDEX idx_api_keys_last_used_at
    ON api_keys(last_used_at);


-- SOURCES
CREATE INDEX idx_sources_tenant_id
    ON sources(tenant_id);

CREATE INDEX idx_sources_status
    ON sources(status);


-- WEBHOOK ENDPOINTS
CREATE INDEX idx_webhook_endpoints_tenant_id
    ON webhook_endpoints(tenant_id);

CREATE INDEX idx_webhook_endpoints_source_id
    ON webhook_endpoints(source_id);


-- DESTINATIONS
CREATE INDEX idx_destinations_tenant_id
    ON destinations(tenant_id);

CREATE INDEX idx_destinations_status
    ON destinations(status);


-- SUBSCRIPTIONS
CREATE INDEX idx_subscriptions_tenant_id
    ON subscriptions(tenant_id);

CREATE INDEX idx_subscriptions_source_id
    ON subscriptions(source_id);

CREATE INDEX idx_subscriptions_destination_id
    ON subscriptions(destination_id);

CREATE INDEX idx_subscriptions_status
    ON subscriptions(status);


-- EVENTS
CREATE INDEX idx_events_tenant_id
    ON events(tenant_id);

CREATE INDEX idx_events_source_id
    ON events(source_id);

CREATE INDEX idx_events_type
    ON events(tenant_id, event_type);

CREATE INDEX idx_events_received_at
    ON events(received_at DESC);

CREATE INDEX idx_events_external_id
    ON events(tenant_id, external_id);


-- DELIVERIES
CREATE INDEX idx_deliveries_tenant_id
    ON deliveries(tenant_id);

CREATE INDEX idx_deliveries_event_id
    ON deliveries(event_id);

CREATE INDEX idx_deliveries_subscription_id
    ON deliveries(subscription_id);

CREATE INDEX idx_deliveries_destination_id
    ON deliveries(destination_id);

CREATE INDEX idx_deliveries_status
    ON deliveries(status);

CREATE INDEX idx_deliveries_retry_queue
    ON deliveries(status, next_attempt_at);


-- DELIVERY ATTEMPTS
CREATE INDEX idx_delivery_attempts_delivery_id
    ON delivery_attempts(delivery_id);

CREATE INDEX idx_delivery_attempts_status
    ON delivery_attempts(status);

CREATE INDEX idx_delivery_attempts_created_at
    ON delivery_attempts(created_at DESC);


-- DEAD LETTER QUEUE
CREATE INDEX idx_dlq_tenant_id
    ON dead_letter_events(tenant_id);

CREATE INDEX idx_dlq_delivery_id
    ON dead_letter_events(delivery_id);

CREATE INDEX idx_dlq_unresolved
    ON dead_letter_events(tenant_id, resolved_at)
    WHERE resolved_at IS NULL;


-- AUDIT LOGS
CREATE INDEX idx_audit_logs_tenant_id
    ON audit_logs(tenant_id);

CREATE INDEX idx_audit_logs_user_id
    ON audit_logs(user_id);

CREATE INDEX idx_audit_logs_resource
    ON audit_logs(resource_type, resource_id);

CREATE INDEX idx_audit_logs_created_at
    ON audit_logs(created_at DESC);


-- USAGE
CREATE INDEX idx_usage_tenant_id
    ON usage_records(tenant_id);

CREATE INDEX idx_usage_period
    ON usage_records(period_start, period_end);


-- ============================================================
-- UPDATED_AT TRIGGER
-- ============================================================

CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER tenants_updated_at
BEFORE UPDATE ON tenants
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();


CREATE TRIGGER users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();


CREATE TRIGGER sources_updated_at
BEFORE UPDATE ON sources
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();


CREATE TRIGGER webhook_endpoints_updated_at
BEFORE UPDATE ON webhook_endpoints
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();


CREATE TRIGGER destinations_updated_at
BEFORE UPDATE ON destinations
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();


CREATE TRIGGER subscriptions_updated_at
BEFORE UPDATE ON subscriptions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();


CREATE TRIGGER deliveries_updated_at
BEFORE UPDATE ON deliveries
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();


CREATE TRIGGER usage_records_updated_at
BEFORE UPDATE ON usage_records
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();
