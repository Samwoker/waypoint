# API Reference — Overview & Conventions

The RelayCore REST API provides complete programmatic control over your webhook infrastructure, event pipelines, routing rules, and delivery traces.

---

## 🌐 Base URL & Versioning

- **Default Local Port**: `3001`
- **Standard Base Path**: `/api/v1` (with backward-compatible aliases on `/v1` and root `/`)
- **Example Ingestion Endpoint**: `http://localhost:3001/hooks/:slug`
- **Example Management Endpoint**: `http://localhost:3001/api/v1/sources`

---

## 🔐 Standard Request Headers

All authenticated management endpoints require:

| Header | Description | Example |
| :--- | :--- | :--- |
| `Authorization` | JWT access token or programmatic API key. | `Authorization: Bearer <TOKEN>` |
| `Content-Type` | MIME type for JSON request bodies. | `Content-Type: application/json` |
| `Accept` | Desired response format. | `Accept: application/json` |

---

## 🚦 Standard HTTP Status Codes

| Status Code | Meaning | Description |
| :--- | :--- | :--- |
| `200 OK` | Success | The request completed successfully and data is returned in the response body. |
| `201 Created` | Created | A new resource (Source, Destination, Subscription, API Key) was created. |
| `202 Accepted` | Accepted | The webhook payload was verified and queued for asynchronous processing. |
| `204 No Content` | No Content | The resource was deleted successfully (e.g. `DELETE /api/v1/sources/:id`). |
| `400 Bad Request` | Validation Error | The request payload contained invalid JSON, missing fields, or invalid slugs. |
| `401 Unauthorized` | Auth Required | Missing, invalid, or expired Bearer token / API key. |
| `403 Forbidden` | Access Denied | The authenticated tenant or user lacks permission for the requested resource. |
| `404 Not Found` | Not Found | The requested resource ID does not exist in the tenant workspace. |
| `409 Conflict` | Conflict | Duplicate resource conflict (e.g., duplicate slug or duplicate subscription). |
| `500 Server Error` | Internal Error | An unexpected database or internal server error occurred. |

---

## 🛑 Standard Error Response Envelope

When an error occurs, RelayCore returns a JSON object:

```json
{
  "error": {
    "status": 400,
    "message": "Invalid source slug. Slugs must contain only lowercase alphanumeric characters and hyphens.",
    "code": "INVALID_SLUG_FORMAT"
  }
}
```

---

## 📑 Keyset Cursor Pagination

Large resource collections (Events, Deliveries, Verification Logs) use **keyset cursor pagination** to deliver fast $O(1)$ database query performance regardless of table size.

### Query Parameters:
- `limit` *(optional, integer)*: Maximum number of items to return (Default: `20`, Max: `100`).
- `cursor` *(optional, string)*: An opaque pagination cursor returned from the previous page's `next_cursor` property.

### Example Paginated Response:
```json
{
  "events": [
    { "id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d", "event_type": "payment.created", ... }
  ],
  "next_cursor": "eyJpZCI6IjliMWRlYjRkLTNiN2QtNGJhZC05YmRkLTJiMGQ3YjNkY2I2ZCIsInRzIjoxNzg4MDA1MjQ1fQ==",
  "has_more": true
}
```

To fetch the next page:
```bash
curl -X GET "http://localhost:3001/api/v1/events?limit=20&cursor=eyJpZCI6IjliMWRlYjRkLTNiN2QtNGJhZC05YmRkLTJiMGQ3YjNkY2I2ZCIsInRzIjoxNzg4MDA1MjQ1fQ==" \
  -H "Authorization: Bearer <TOKEN>"
```

---

## ⏭️ API Endpoint Sections

- [Authentication & JWT Tokens](./authentication.md)
- [Programmatic API Keys](./api-keys.md)
- [Inbound Sources](./sources.md)
- [Target Destinations](./destinations.md)
- [Routing Subscriptions](./subscriptions.md)
- [Events Stream & Ingestion](./events.md)
- [Deliveries & Traces](./deliveries.md)
- [Dead Letter Queue (DLQ)](./dlq.md)
- [Transformation Engine](./transformations.md)
- [Statistics & Telemetry](./stats.md)
- [Tenants & Quotas](./tenants.md)
