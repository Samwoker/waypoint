# Installation & Local Setup

This guide walks you through setting up and running **RelayCore (Waypoint)** locally for development and testing.

---

## 📋 Prerequisites

Ensure you have the following installed on your machine:

- **Rust & Cargo**: Version `1.80+` (Install via [rustup.rs](https://rustup.rs/))
- **Node.js & npm**: Node.js `v20+` or `v24+` (For UI and SDK integrations)
- **PostgreSQL**: Version `15+` or `16+`
- **Redis**: Version `7.0+`
- **Docker & Docker Compose**: (Optional, recommended for rapid containerized setup)
- **sqlx-cli**: (Optional, for database migrations) `cargo install sqlx-cli --no-default-features --features postgres`

---

## 🚀 Option 1: Quickstart with Docker Compose (Recommended)

The fastest way to spin up the entire RelayCore stack (PostgreSQL, Redis, API Gateway, Worker Pool, and Frontend Dashboard):

```bash
# 1. Clone the repository
git clone https://github.com/Samwoker/waypoint.git
cd waypoint

# 2. Copy the environment template
cp .env.example .env

# 3. Start all services in the background
docker-compose up -d

# 4. Verify running containers
docker-compose ps
```

Once running:
- **API Server**: Available at `http://localhost:3001`
- **Frontend Dashboard**: Available at `http://localhost:5175`
- **PostgreSQL**: Listening on `localhost:5432`
- **Redis**: Listening on `localhost:6379`

---

## 🛠️ Option 2: Native Local Development

### 1. Configure Environment Variables

Create `.env` in the repository root:

```bash
cp .env.example .env
```

Default configuration in `.env`:
```dotenv
DATABASE_URL=postgres://postgres:postgres@localhost:5432/webhook_relay
REDIS_URL=redis://127.0.0.1:6379
API_PORT=3001
API_HOST=0.0.0.0
JWT_SECRET=super-secret-jwt-key-minimum-32-characters-long!
WORKER_CONCURRENCY=10
WORKER_POLL_INTERVAL_MS=100
LOG_LEVEL=info
```

### 2. Start PostgreSQL & Redis

If using Docker for databases only:
```bash
docker run -d --name relay-pg -p 5432:5432 -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=webhook_relay postgres:16-alpine
docker run -d --name relay-redis -p 6379:6379 redis:7-alpine
```

### 3. Run Database Migrations

Apply SQL migrations located in `migrations/`:
```bash
# Using sqlx-cli
sqlx database setup

# Or using psql directly
for file in migrations/*.sql; do
    psql "$DATABASE_URL" -f "$file"
done
```

### 4. Build and Run Backend Services

In the repository root:

```bash
# Terminal 1: Run the Axum API Server
cargo run -p api

# Terminal 2: Run the Background Delivery Worker
cargo run -p worker
```

### 5. Build and Run Frontend Dashboard

```bash
cd ui
npm install
npm run dev
```

The frontend will start at `http://localhost:5175`.

---

## ✅ Verifying Installation

Execute a health check request against the API server:

```bash
curl -s http://localhost:3001/healthz
```

Expected output:
```json
{
  "db": "ok",
  "queue": "ok"
}
```

---

## 🧪 Running Automated Tests

Run the full workspace unit, integration, and tenant isolation test suite:

```bash
cargo test --workspace
```

Run frontend production build & lint checks:

```bash
cd ui
npm run lint
npm run build
```

---

## ⏭️ Next Steps

- Follow the [5-Minute Quickstart Tutorial](./quickstart.md) to ingest your first webhook!
- Check the [Environment Variables Reference](./environment-variables.md).
