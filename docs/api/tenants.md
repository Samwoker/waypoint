# Tenants & Quota Management API

The Tenants API manages multi-tenant workspace metadata, organization settings, and event ingestion quota meters.

---

## 1. List Tenants (Platform Admin)

`GET /api/v1/tenants`

Retrieves all tenants across the platform.

### Success Response (`200 OK`):
```json
[
  {
    "id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
    "name": "Acme Payments",
    "slug": "acme-payments-8899",
    "created_at": "2026-08-28T12:00:00Z",
    "updated_at": "2026-08-28T12:00:00Z"
  }
]
```

---

## 2. Get Tenant Ingestion Usage & Plan Quotas

`GET /api/v1/tenants/:id/usage?period=30d`

Returns billing cycle usage statistics, total events ingested, outbound delivery dispatches, and daily volume breakdown.

### Success Response (`200 OK`):
```json
{
  "tenant_id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
  "total_events": 48250,
  "total_delivery_attempts": 96500,
  "period": "30d",
  "daily_events": [
    { "date": "2026-08-01", "count": 1450 },
    { "date": "2026-08-02", "count": 1620 },
    { "date": "2026-08-03", "count": 1580 }
  ]
}
```

---

## 3. Update Tenant Organization Name

`PUT /api/v1/tenants/:id`

Updates the organization display name.

### Request Body:
```json
{
  "name": "Acme Global Payments Corp"
}
```

### Success Response (`200 OK`):
```json
{
  "id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
  "name": "Acme Global Payments Corp",
  "slug": "acme-payments-8899",
  "updated_at": "2026-08-28T15:00:00Z"
}
```
