# Target Destinations API

Target Destinations define downstream external endpoints where RelayCore delivers validated webhooks, complete with timeout thresholds, retry policies, and circuit breaker health states.

---

## 1. List Destinations

`GET /api/v1/destinations`

Retrieves all target destination endpoints registered for the tenant.

### Success Response (`200 OK`):
```json
[
  {
    "id": "9946ffcf-e7c7-427b-951c-7b3e0e482855",
    "tenant_id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
    "name": "Internal Billing Receiver",
    "url": "https://api.example.com/webhooks/billing",
    "status": "active",
    "is_active": true,
    "consecutive_failures": 0,
    "max_retries": 5,
    "timeout_ms": 5000,
    "retry_backoff_strategy": "exponential",
    "rate_limit_rps": 100,
    "created_at": "2026-08-28T12:00:00Z",
    "updated_at": "2026-08-28T12:00:00Z"
  }
]
```

---

## 2. Create Target Destination

`POST /api/v1/destinations`

Registers a new target endpoint.

### Request Body:
```json
{
  "name": "Customer Data Sync",
  "url": "https://api.example.com/webhooks/sync",
  "timeout_ms": 5000,
  "max_retries": 5,
  "rate_limit_rps": 100
}
```

### Success Response (`201 Created`):
```json
{
  "id": "9946ffcf-e7c7-427b-951c-7b3e0e482855",
  "tenant_id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
  "name": "Customer Data Sync",
  "url": "https://api.example.com/webhooks/sync",
  "status": "active",
  "is_active": true,
  "consecutive_failures": 0,
  "max_retries": 5,
  "timeout_ms": 5000,
  "retry_backoff_strategy": "exponential",
  "rate_limit_rps": 100,
  "created_at": "2026-08-28T12:00:00Z",
  "updated_at": "2026-08-28T12:00:00Z"
}
```

---

## 3. Destination Health Diagnostics

`GET /api/v1/destinations/:id/health`

Returns deep health diagnostics, circuit breaker state, success rate over the last 24 hours, and timestamps of recent successes and failures.

### Success Response (`200 OK`):
```json
{
  "destination_id": "9946ffcf-e7c7-427b-951c-7b3e0e482855",
  "status": "healthy",
  "is_active": true,
  "circuit_state": "closed",
  "consecutive_failures": 0,
  "success_rate_24h": 0.998,
  "total_deliveries_24h": 14250,
  "failed_deliveries_24h": 28,
  "avg_latency_ms": 46.5,
  "p95_latency_ms": 128.0,
  "last_successful_delivery_at": "2026-08-28T14:32:10Z",
  "last_failed_delivery_at": "2026-08-28T09:15:22Z"
}
```

---

## 4. Test Destination Endpoint

`POST /api/v1/destinations/:id/test`

Dispatches an immediate probe webhook payload to the destination URL and returns the remote server's response status code, latency, and response headers.

### Success Response (`200 OK`):
```json
{
  "success": true,
  "status_code": 200,
  "latency_ms": 78,
  "response_headers": {
    "content-type": "application/json",
    "server": "nginx/1.24"
  },
  "response_body": "{\"status\":\"ok\"}"
}
```

---

## 5. Pause & Resume Deliveries

### Pause: `POST /api/v1/destinations/:id/pause`
Temporarily halts outbound deliveries to this destination.

### Resume: `POST /api/v1/destinations/:id/resume`
Resumes deliveries and resets an open circuit breaker back to `closed`.

---

## 6. Delete Destination

`DELETE /api/v1/destinations/:id`

Permanently removes the destination endpoint and any associated subscriptions.
