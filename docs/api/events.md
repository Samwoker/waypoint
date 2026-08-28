# Events Stream & Ingestion API

Events represent immutable webhook payloads received through Inbound Sources.

---

## 1. Ingest Public Webhook

`POST /hooks/:slug`

The public unauthenticated webhook ingestion entrypoint.

### Headers:
- `Content-Type: application/json`
- `X-Event-Type: <event_type>` *(optional if event type is embedded in JSON body)*
- `Stripe-Signature` / `X-Hub-Signature-256` / `X-Signature` *(if signature verification is enabled)*

### Request Body:
```json
{
  "event": "payment.succeeded",
  "data": {
    "id": "ch_3MjjkwLkdIwHu7ix",
    "amount": 2999,
    "currency": "usd"
  }
}
```

### Success Response (`202 Accepted`):
```json
{
  "id": "d2344500-3731-4c8d-9763-9486789e31cb",
  "event_type": "payment.succeeded",
  "status": "received",
  "created_at": "2026-08-28T14:30:00Z"
}
```

---

## 2. List Ingested Events (Keyset Pagination)

`GET /api/v1/events?limit=20&cursor=<opaque_cursor>`

Retrieves a paginated list of ingested events for the tenant.

### Success Response (`200 OK`):
```json
{
  "events": [
    {
      "id": "d2344500-3731-4c8d-9763-9486789e31cb",
      "tenant_id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
      "source_id": "2e38d7c0-ebaf-4441-b9da-5c35ff6e4a50",
      "source_name": "Stripe Production",
      "event_type": "payment.succeeded",
      "status": "delivered",
      "received_at": "2026-08-28T14:30:00Z",
      "created_at": "2026-08-28T14:30:00Z"
    }
  ],
  "next_cursor": "eyJpZCI6ImQyMzQ0NTAwLTM3MzEtNGM4ZC05NzYzLTk0ODY3ODllMzFjYiIsInRzIjoxNzg4MDA1MjQ1fQ==",
  "has_more": true
}
```

---

## 3. Get Event Details & Delivery Summary

`GET /api/v1/events/:id`

Retrieves event metadata and a summary of generated deliveries.

### Success Response (`200 OK`):
```json
{
  "id": "d2344500-3731-4c8d-9763-9486789e31cb",
  "tenant_id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
  "source_id": "2e38d7c0-ebaf-4441-b9da-5c35ff6e4a50",
  "event_type": "payment.succeeded",
  "status": "delivered",
  "delivery_summary": {
    "total": 2,
    "delivered": 2,
    "failed": 0,
    "pending": 0
  },
  "received_at": "2026-08-28T14:30:00Z",
  "created_at": "2026-08-28T14:30:00Z"
}
```

---

## 4. Get Sensitive Raw Payload & Inbound Headers

`GET /api/v1/events/:id/raw`

Retrieves the exact byte-for-byte raw JSON payload and HTTP headers captured upon arrival.

### Success Response (`200 OK`):
```json
{
  "event_id": "d2344500-3731-4c8d-9763-9486789e31cb",
  "payload": {
    "event": "payment.succeeded",
    "data": {
      "id": "ch_3MjjkwLkdIwHu7ix",
      "amount": 2999,
      "currency": "usd"
    }
  },
  "headers": {
    "content-type": "application/json",
    "stripe-signature": "t=1614555845,v1=5257a869e7ecebeda32affa62cdca3fa51cad7e77a0e56ff536d0ce8e108d8bd",
    "user-agent": "Stripe/1.0 (+https://stripe.com/docs/webhooks)"
  }
}
```

---

## 5. Get Event Fan-Out Deliveries

`GET /api/v1/events/:id/deliveries`

Returns all deliveries dispatched for this event across matching subscriptions.

### Success Response (`200 OK`):
```json
[
  {
    "id": "5f6e7d8c-9a0b-1c2d-3e4f-5a6b7c8d9e0f",
    "event_id": "d2344500-3731-4c8d-9763-9486789e31cb",
    "destination_id": "9946ffcf-e7c7-427b-951c-7b3e0e482855",
    "destination_name": "Internal Billing Receiver",
    "destination_url": "https://api.example.com/webhooks/billing",
    "status": "delivered",
    "attempt_count": 1,
    "max_attempts": 5,
    "last_attempt_at": "2026-08-28T14:30:02Z",
    "created_at": "2026-08-28T14:30:00Z"
  }
]
```

---

## 6. Replay Event

`POST /api/v1/events/:id/replay`

Re-evaluates subscription rules for this event and creates new delivery dispatch tasks for all matching destinations.

### Success Response (`200 OK`):
```json
{
  "event_id": "d2344500-3731-4c8d-9763-9486789e31cb",
  "status": "replayed",
  "new_deliveries_count": 2
}
```

---

## 7. Batch Replay Events

`POST /api/v1/events/replay-batch`

Replays a list of event IDs in a single batch operation.

### Request Body:
```json
{
  "event_ids": [
    "d2344500-3731-4c8d-9763-9486789e31cb",
    "e3455611-4842-5d9e-0874-0597890f42dc"
  ]
}
```

### Success Response (`200 OK`):
```json
{
  "replayed_events_count": 2,
  "deliveries_created_count": 4
}
```
