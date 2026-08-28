# System Architecture

RelayCore is built in **Rust** using asynchronous Tokio runtimes, Axum HTTP routers, SQLx for connection-pooled PostgreSQL persistence, and Redis for distributed message queuing.

---

## 🏛️ High-Level System Architecture

```mermaid
flowchart TB
    subgraph Ingestion Layer
        Client["External Webhook Client<br/>(e.g., Stripe, Shopify)"]
        API["Axum HTTP Ingestion Server<br/>(crates/api)"]
    end

    subgraph Storage & Queue Layer
        PG[(PostgreSQL Database<br/>Tenant Data, Events, Deliveries)]
        Redis[(Redis Streams / Queue<br/>Delivery Work Items)]
    end

    subgraph Worker & Egress Layer
        WorkerPool["Background Worker Pool<br/>(crates/worker)"]
        Worker1["Tokio Worker Task 1"]
        Worker2["Tokio Worker Task 2"]
        WorkerN["Tokio Worker Task N"]
        WorkerPool --- Worker1 & Worker2 & WorkerN
    end

    subgraph Downstream Destinations
        DestA["Destination Service A<br/>(Billing API)"]
        DestB["Destination Service B<br/>(Analytics API)"]
    end

    Client -->|1. POST /hooks/:slug| API
    API -->|2. Constant-Time HMAC Verify| API
    API -->|3. Persist Event & Match Subs| PG
    API -->|4. Push Delivery Tasks| Redis
    API -->|5. 202 Accepted| Client

    Redis -->|6. Fetch Ready Jobs| WorkerPool
    Worker1 -->|7. Transform & HTTP POST| DestA
    Worker2 -->|7. Transform & HTTP POST| DestB
    Worker1 & Worker2 -->|8. Record Attempt Traces & Status| PG
```

---

## ⚙️ Component Breakdown

### 1. Ingestion API Server (`crates/api`)
- **Technology**: Rust + Axum + Tower HTTP.
- **Responsibility**: Inbound webhook receipt, cryptographic signature verification, tenant authorization, REST administration APIs, and health/Prometheus metric reporting.
- **Performance**: Returns `202 Accepted` within `<5ms`. No heavy downstream HTTP requests or complex transformations are executed in the HTTP request thread.

### 2. Domain & Routing Logic (`crates/domain` & `crates/core`)
- **Responsibility**: Encapsulates business logic, subscription matching algorithms, wildcard event type evaluation (`order.*` matches `order.created`), cryptographic signature engines (HMAC-SHA256, Stripe v1, GitHub, Shopify), and circuit breaker state calculation.

### 3. Data Layer (`crates/data`)
- **Technology**: SQLx + PostgreSQL + Redis.
- **Responsibility**: Strong consistency, multi-tenant row-level partitioning by `tenant_id`, atomic event ingestion transactions, delivery queueing, and historical attempt tracing.

### 4. Background Delivery Worker (`crates/worker`)
- **Technology**: Tokio Asynchronous Task Pool + Reqwest HTTP Client.
- **Responsibility**:
  - Pulls delivery work items from Redis.
  - Checks target destination circuit breaker state.
  - Applies JSONPath payload transformations.
  - Dispatches HTTP POST requests to customer endpoints with configurable timeouts (e.g. 5000ms).
  - Evaluates response HTTP status codes (2xx = delivered, 4xx/5xx/timeout = retry or dead letter).
  - Computes exponential backoff with randomized jitter.
  - Quarantines permanently failed deliveries to the Dead Letter Queue (DLQ).

---

## ⏱️ Synchronous vs. Asynchronous Operations

| Phase | Execution Mode | Executed By | SLA / Timeout |
| :--- | :--- | :--- | :--- |
| **Inbound Webhook Receipt** | Synchronous | `crates/api` | `< 5ms` |
| **Signature Validation** | Synchronous | `crates/core` | `< 1ms` (Constant Time) |
| **Event Persistence** | Synchronous | PostgreSQL | `< 3ms` |
| **Subscription Fan-Out** | Synchronous | Ingestion Transaction | `< 2ms` |
| **Delivery Dispatch** | Asynchronous | `crates/worker` | Up to configured `timeout_ms` |
| **Retry Backoff Delay** | Asynchronous | Redis Delayed Queue | Configurable (e.g. $2^n \times 10s$) |
| **Payload Transformation** | Asynchronous | `crates/worker` | `< 1ms` |
| **Circuit Breaker Check** | Asynchronous | `crates/worker` | In-memory atomic check |

---

## 🗄️ Database Entity-Relationship Model

```mermaid
erDiagram
    TENANTS ||--o{ USERS : contains
    TENANTS ||--o{ API_KEYS : issues
    TENANTS ||--o{ SOURCES : owns
    TENANTS ||--o{ DESTINATIONS : owns
    TENANTS ||--o{ EVENTS : receives

    SOURCES ||--o{ EVENTS : ingests
    SOURCES ||--o{ SUBSCRIPTIONS : routes
    DESTINATIONS ||--o{ SUBSCRIPTIONS : receives
    SUBSCRIPTIONS ||--o{ TRANSFORMATIONS : transforms

    EVENTS ||--o{ DELIVERIES : fans_out
    DESTINATIONS ||--o{ DELIVERIES : delivers_to
    DELIVERIES ||--o{ DELIVERY_ATTEMPTS : logs
    DELIVERIES ||--o| DLQ : quarantines
```

---

## ⏭️ Next Steps

- Proceed to the [Installation & Local Setup](../getting-started/installation.md).
- Follow the [5-Minute Quickstart](../getting-started/quickstart.md).
- Review [Environment Variables](../getting-started/environment-variables.md).
