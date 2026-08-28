# Dead Letter Queue (DLQ) API

The Dead Letter Queue quarantines failed deliveries that have exhausted their retry policies, preventing worker pipeline blockage while preserving full debugging traces for manual or bulk recovery.

---

## 1. List Quarantined DLQ Deliveries

`GET /api/v1/dlq?limit=50`

Retrieves all deliveries currently quarantined in the DLQ for the authenticated tenant.

### Success Response (`200 OK`):
```json
{
  "items": [
    {
      "delivery_id": "5f6e7d8c-9a0b-1c2d-3e4f-5a6b7c8d9e0f",
      "event_id": "d2344500-3731-4c8d-9763-9486789e31cb",
      "event_type": "payment.succeeded",
      "destination_name": "Internal Billing Receiver",
      "attempt_count": 5,
      "max_attempts": 5,
      "last_error": "Connection timed out after 5000ms",
      "dead_lettered_at": "2026-08-28T14:35:00Z"
    }
  ],
  "has_more": false
}
```

---

## 2. Requeue Single DLQ Delivery

`POST /api/v1/dlq/:id/requeue`

Moves a quarantined delivery back into the active Redis worker queue for immediate retry.

### Success Response (`200 OK`):
```json
{
  "delivery_id": "5f6e7d8c-9a0b-1c2d-3e4f-5a6b7c8d9e0f",
  "status": "requeued",
  "requeued_at": "2026-08-28T14:40:00Z"
}
```

---

## 3. Bulk Retry All Quarantined Items

`POST /api/v1/dlq/retry-all`

Bulk re-enqueues every dead-lettered delivery across the tenant in a single atomic database operation.

### Success Response (`200 OK`):
```json
{
  "replayed_count": 14
}
```

---

## 4. Discard Dead-Lettered Delivery

`DELETE /api/v1/dlq/:id`

Marks a delivery as permanently `discarded`. The underlying event and historical attempt traces remain in PostgreSQL for audit compliance, but no further retries will be scheduled.

### Success Response (`204 No Content`):
Empty response body.
