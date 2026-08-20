# Waypoint / RelayCore

High-performance, multi-tenant webhook and event delivery platform built with Rust (Axum + Tokio + SQLx + PostgreSQL + Redis).

## Features

- **Multi-Tenant Architecture**: Strict tenant isolation across all endpoints and database queries.
- **Event Ingestion**: High-throughput asynchronous event ingestion with idempotency deduplication and Redis Streams.
- **Webhook Sources**: Inbound webhook source management with AES-256-GCM encrypted secrets, automated 32-byte secret generation, and HMAC verification.
- **Webhook Destinations**: Outbound webhook delivery destinations with SSRF protection, rate limiting, and configurable retry policies.
- **Subscriptions & Filtering**: Source-to-destination routing rules with event type matching and JSON payload filtering.
- **Deliveries & DLQ**: Resilient webhook dispatching, attempt history, and dead-letter queue tracking.
- **API Keys & Authentication**: Secure cryptographic API key lifecycle management (SHA-256 hashed storage, secret redaction, and logical revocation).
- **Observability**: Built-in health checks (`/v1/health`), operational Prometheus-style metrics (`/v1/metrics`), and tenant audit logs (`/v1/audit-logs`).
- **Tenant Usage Aggregation**: Native PostgreSQL day-by-day volume and delivery attempt aggregations (`GET /tenants/{tenant_id}/usage`).

## Architecture

```text
waypoint/
├── crates/
│   ├── api/        # Axum HTTP API handlers and routing
│   ├── core/       # Configuration, cryptographic primitives, and error types
│   ├── data/       # PostgreSQL models, migrations, and SQLx repositories
│   ├── domain/     # Business logic, DTOs, and domain services
│   └── worker/     # Async delivery poller and Redis fanout consumer
└── migrations/     # PostgreSQL schema migrations
```

## Running Locally

### Prerequisites
- Rust (1.75+)
- PostgreSQL (15+)
- Redis (7+)

### Setup
1. Copy `.env.example` to `.env` and configure credentials:
   ```bash
   cp .env.example .env
   ```
2. Run database migrations and start the server:
   ```bash
   cargo run -p api
   ```

### Running Tests
```bash
cargo test --workspace
```
