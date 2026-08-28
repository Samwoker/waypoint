# Deliveries & Attempt Traces API

Deliveries represent outbound webhook dispatch tasks and record chronological attempt traces with execution latencies and response snippets.

---

## 1. List Deliveries (Filtered & Paginated)

`GET /api/v1/deliveries?status=failed&limit=20&cursor=<opaque_cursor>`

Retrieves deliveries across the tenant with optional status filtering.

### Query Parameters:
- `status` *(optional)*: Filter by delivery state: `all`, `delivered`, `failed`, `pending`, `dead_letter`, `retry`.
- `limit` *(optional)*: Page size (Default: `20`).
- `cursor` *(optional)*: Keyset cursor.

### Success Response (`200 OK`):
```json
{
  "deliveries": [
    {
      "id": "5f6e7d8c-9a0b-1c2d-3e4f-5a6b7c8d9e0f",
      "tenant_id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
      "event_id": "d2344500-3731-4c8d-9763-9486789e31cb",
      "destination_id": "9946ffcf-e7c7-427b-951c-7b3e0e482855",
      "destination_name": "Internal Billing Receiver",
      "status": "delivered",
      "attempt_count": 1,
      "max_attempts": 5,
      "created_at": "2026-08-28T14:30:00Z"
    }
  ],
  "next_cursor": "eyJpZCI6IjVmNmU3ZDhjLTlhMGItMWMyZC0zZTRmLTVhNmI3YzhkOWUwZiJ9",
  "has_more": false
}
```

---

## 2. Get Delivery Details

`GET /api/v1/deliveries/:id`

Retrieves delivery metadata along with linked Event and Destination details.

### Success Response (`200 OK`):
```json
{
  "id": "5f6e7d8c-9a0b-1c2d-3e4f-5a6b7c8d9e0f",
  "tenant_id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
  "event_id": "d2344500-3731-4c8d-9763-9486789e31cb",
  "event_type": "payment.succeeded",
  "source_name": "Stripe Production",
  "destination_id": "9946ffcf-e7c7-427b-951c-7b3e0e482855",
  "destination_name": "Internal Billing Receiver",
  "destination_url": "https://api.example.com/webhooks/billing",
  "status": "delivered",
  "attempt_count": 1,
  "max_attempts": 5,
  "next_retry_at": null,
  "created_at": "2026-08-28T14:30:00Z",
  "updated_at": "2026-08-28T14:30:02Z"
}
```

---

## 3. Get Chronological Attempt Traces

`GET /api/v1/deliveries/:id/attempts`

Returns the sequential trace of every HTTP POST attempt made for this delivery.

### Success Response (`200 OK`):
```json
[
  {
    "id": "3a4b5c6d-7e8f-9a0b-1c2d-3e4f5a6b7c8d",
    "delivery_id": "5f6e7d8c-9a0b-1c2d-3e4f-5a6b7c8d9e0f",
    "attempt_number": 1,
    "status_code": 503,
    "latency_ms": 142,
    "error_message": "Remote server returned 503 Service Unavailable",
    "response_body": "{\"error\":\"database undergoing maintenance\"}",
    "executed_at": "2026-08-28T14:30:01Z"
  },
  {
    "id": "4b5c6d7e-8f9a-0b1c-2d3e-4f5a6b7c8d9e",
    "delivery_id": "5f6e7d8c-9a0b-1c2d-3e4f-5a6b7c8d9e0f",
    "attempt_number": 2,
    "status_code": 200,
    "latency_ms": 68,
    "error_message": null,
    "response_body": "{\"success\":true,\"received_id\":\"ch_3MjjkwLkdIwHu7ix\"}",
    "executed_at": "2026-08-28T14:30:12Z"
  }
]
```

---

## 4. Replay Delivery

`POST /api/v1/deliveries/:id/replay`

Re-enqueues a specific delivery for immediate dispatch by background workers, resetting its status back to `pending`.

### Success Response (`200 OK`):
```json
{
  "delivery_id": "5f6e7d8c-9a0b-1c2d-3e4f-5a6b7c8d9e0f",
  "status": "requeued"
}
```
