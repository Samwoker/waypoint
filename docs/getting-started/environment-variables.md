# Environment Variables Reference

This document lists every environment variable utilized by **RelayCore (Waypoint)** across the API Server (`crates/api`), Background Worker (`crates/worker`), Core Cryptographic Engine (`crates/core`), and Frontend (`ui`).

---

## 🗄️ Core Database & Queue Configuration

| Variable | Required | Default | Description |
| :--- | :--- | :--- | :--- |
| `DATABASE_URL` | **Yes** | `postgres://postgres:postgres@localhost:5432/webhook_relay` | PostgreSQL connection string including credentials and database name. |
| `REDIS_URL` | **Yes** | `redis://127.0.0.1:6379` | Redis connection URL for distributed worker job queues and delayed retry scheduling. |
| `DATABASE_MAX_CONNECTIONS` | No | `20` | Maximum size of the SQLx PostgreSQL connection pool. |
| `DATABASE_MIN_CONNECTIONS` | No | `5` | Minimum idle connections retained in the PostgreSQL pool. |
| `DATABASE_ACQUIRE_TIMEOUT_SECS` | No | `30` | Maximum time in seconds to wait when acquiring a connection from the pool. |

---

## 🌐 API Server Configuration (`crates/api`)

| Variable | Required | Default | Description |
| :--- | :--- | :--- | :--- |
| `API_PORT` | No | `3001` | TCP port on which the Axum HTTP REST server binds. |
| `API_HOST` | No | `0.0.0.0` | IP interface address for the API server (use `0.0.0.0` in Docker/Kubernetes). |
| `JWT_SECRET` | **Yes** | — | Cryptographic secret key used to sign and verify user JWT Bearer access & refresh tokens (minimum 32 chars). |
| `JWT_EXPIRATION_SECS` | No | `86400` (24h) | Time-to-live for access tokens in seconds. |
| `REFRESH_TOKEN_EXPIRATION_SECS` | No | `2592000` (30d) | Time-to-live for refresh tokens in seconds. |
| `LOG_LEVEL` | No | `info` | Logging verbosity filter (`trace`, `debug`, `info`, `warn`, `error`). |
| `CORS_ALLOWED_ORIGINS` | No | `*` | Comma-separated list of allowed browser origins for CORS preflight headers. |

---

## ⚙️ Background Worker Configuration (`crates/worker`)

| Variable | Required | Default | Description |
| :--- | :--- | :--- | :--- |
| `WORKER_CONCURRENCY` | No | `10` | Number of simultaneous Tokio asynchronous worker task threads polling and executing delivery jobs. |
| `WORKER_POLL_INTERVAL_MS` | No | `100` | Polling backoff interval in milliseconds when the Redis queue is empty. |
| `WORKER_REQUEST_TIMEOUT_MS` | No | `5000` | Global fallback HTTP client request timeout for outbound destination dispatches. |
| `CIRCUIT_BREAKER_FAILURE_THRESHOLD` | No | `5` | Number of consecutive delivery failures before a destination circuit breaker trips `open`. |
| `CIRCUIT_BREAKER_COOLDOWN_SECS` | No | `60` | Duration in seconds an open circuit remains before transitioning to `half-open` probe state. |

---

## 🔐 Security & Secret Encryption

| Variable | Required | Default | Description |
| :--- | :--- | :--- | :--- |
| `ENCRYPTION_KEY` | No | Built-in fallback | 32-byte AES-256-GCM symmetric key used to encrypt inbound HMAC signing secrets at rest in PostgreSQL. |
| `ENFORCE_HTTPS_IN_PRODUCTION` | No | `false` | When set to `true`, rejects any non-HTTPS target destination URLs. |

---

## 💻 Frontend Configuration (`ui`)

| Variable | Required | Default | Description |
| :--- | :--- | :--- | :--- |
| `VITE_API_BASE_URL` | No | `""` (Relative proxy) | Base URL of the RelayCore API server (e.g. `http://localhost:3001` or `https://api.relaycore.dev`). |
| `PORT` | No | `5175` | Local Vite development server port. |

---

## 📝 Example `.env` File

```dotenv
# Database & Redis
DATABASE_URL=postgres://postgres:postgres@localhost:5432/webhook_relay
REDIS_URL=redis://127.0.0.1:6379
DATABASE_MAX_CONNECTIONS=25

# API Server
API_HOST=0.0.0.0
API_PORT=3001
JWT_SECRET=production-grade-jwt-secret-key-32-characters-minimum!
LOG_LEVEL=info

# Background Worker
WORKER_CONCURRENCY=20
WORKER_POLL_INTERVAL_MS=50
CIRCUIT_BREAKER_FAILURE_THRESHOLD=5
CIRCUIT_BREAKER_COOLDOWN_SECS=60
```
