# Core Concepts & Mental Model

RelayCore is built around an operational pipeline designed for reliable, multi-tenant webhook ingestion, verification, and fan-out delivery.

---

## 🏛️ Entity Relationships Overview

```text
TENANT (Workspace / Organization)
 ├── Users (Admins, Members)
 ├── Programmatic API Keys (read_only, full)
 ├── Inbound Sources
 │     └── Events (Ingested Webhook Payloads)
 │            └── Deliveries (Outbound Forwarding Dispatches)
 │                   └── Delivery Attempts (Chronological HTTP traces)
 ├── Target Destinations (External Servers)
 ├── Subscriptions (Routing Rules: Source -> Destination)
 └── Dead Letter Queue (Quarantined Deliveries)
```

---

## 🔑 Core Concepts Explained

### 1. Tenant
- **What it is**: An isolated workspace representing an organization or project.
- **Why it exists**: Guarantees total data segregation. Resources (Sources, Events, Subscriptions, API keys, and Telemetry) are scoped strictly by `tenant_id`. Cross-tenant data access is rejected at both the middleware and database levels.

### 2. Inbound Source
- **What it is**: A dedicated entrypoint configuration for an upstream webhook provider (e.g. Stripe, GitHub, Shopify, or Custom microservice).
- **Contains**: Unique URL slug (`/hooks/:slug`), provider type, cryptographic verification type (`generic_hmac`, `stripe`, `github`, `shopify`, `none`), signing secret, and tolerance window in seconds.
- **Role**: Receives public HTTP POST requests, validates incoming signatures in constant time, and emits ingested `Events`.

### 3. Event
- **What it is**: An immutable record of an event ingested through an Inbound Source.
- **Contains**: Unique Event UUID, Source ID, Tenant ID, Event Type (e.g., `payment.created`, `pull_request.opened`), raw payload, inbound HTTP headers, and ingestion timestamp.
- **Role**: Serves as the authoritative source of truth. Once created, an event can fan out into multiple `Deliveries` and can be replayed repeatedly without altering original payload data.

### 4. Target Destination
- **What it is**: A downstream receiver URL registered by your team to receive forwarded webhooks (e.g. `https://api.example.com/webhooks/billing`).
- **Contains**: Target URL, timeout thresholds (`timeout_ms`), max retry limit (`max_retries`), rate limits (`rate_limit_rps`), circuit breaker status (`active` / `circuit_open`), and consecutive failure counter.
- **Role**: The receiving endpoint where background workers deliver payloads.

### 5. Routing Subscription
- **What it is**: A declarative rule that connects an **Inbound Source** to a **Target Destination**.
- **Contains**: Source ID, Destination ID, Event Type wildcard filter array (e.g. `["payment.*", "customer.created"]`), and active/paused state.
- **Role**: Controls fan-out. When an Event arrives at a Source, RelayCore checks all active Subscriptions for matching event type patterns and creates a `Delivery` for each matching Destination.

### 6. Delivery
- **What it is**: The system's execution task to forward a specific `Event` to a specific `Destination`.
- **Status Lifecycle**:
  - `pending` $\to$ `processing` $\to$ `delivered` (Success)
  - `pending` $\to$ `processing` $\to$ `retry` (Backoff scheduled)
  - `retry` $\to$ `failed` / `dead_letter` (Retry budget exhausted)
  - `discarded` (Permanently abandoned by operator)
- **Role**: Tracks outbound progress independently for each destination.

### 7. Delivery Attempt
- **What it is**: An individual HTTP POST request made by a background worker for a delivery.
- **Contains**: Attempt sequence number (1, 2, 3...), timestamp, execution duration in milliseconds, HTTP response status code (e.g. 200, 500, 504), error message (if connection timed out or failed DNS), and truncated response body snippet.

### 8. Dead Letter Queue (DLQ)
- **What it is**: A quarantine storage area for deliveries that have failed all retry attempts.
- **Role**: Prevents failing deliveries from blocking the worker pool while allowing operators to inspect root causes, fix downstream bugs, and trigger 1-click single or bulk replays.

### 9. Circuit Breaker
- **What it is**: An automated fault tolerance mechanism for each Destination.
- **Role**: If a destination fails repeatedly (e.g., 5 consecutive 5xx errors or timeouts), the circuit trips `open`, immediately pausing new deliveries to that destination to prevent cascading system degradation.

---

## ⏭️ Next Steps

- Trace [The Complete Event Lifecycle](./event-lifecycle.md).
- Understand [Retries & Exponential Backoff](./retry-and-backoff.md).
- Learn about [Circuit Breakers & Fault Tolerance](./circuit-breaker.md).
