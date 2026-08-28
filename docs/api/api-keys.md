# Programmatic API Keys API

Programmatic API keys allow backend services, CI/CD deployment pipelines, and external applications to authenticate with RelayCore without requiring interactive user logins.

---

## 1. List API Keys

`GET /api/v1/api-keys`

Retrieves all active API keys issued for the authenticated tenant. Keys are returned in masked format (e.g. `rc_live_ab12...`). Full plaintext keys are never returned in list endpoints.

### Headers:
`Authorization: Bearer <TOKEN>`

### Success Response (`200 OK`):
```json
[
  {
    "id": "7b8f9e01-2a3b-4c5d-6e7f-8a9b0c1d2e3f",
    "name": "CI Deployment Pipeline",
    "key_prefix": "rc_live_ab12",
    "last_used_at": "2026-08-28T14:20:00Z",
    "expires_at": "2026-11-28T14:20:00Z",
    "created_at": "2026-08-28T14:20:00Z"
  }
]
```

---

## 2. Create API Key

`POST /api/v1/api-keys`

Generates a new scoped API key.

> **CRITICAL SECURITY NOTE**: The full plaintext API key (`raw_key`) is returned **EXACTLY ONCE** in this creation response. Store it securely in your secret manager. RelayCore hashes keys with SHA-256 before storage and cannot retrieve the raw key again.

### Request Body:
```json
{
  "name": "Stripe Ingestion Microservice",
  "expires_at": "2026-12-31T23:59:59Z"
}
```

### Success Response (`201 Created`):
```json
{
  "id": "7b8f9e01-2a3b-4c5d-6e7f-8a9b0c1d2e3f",
  "name": "Stripe Ingestion Microservice",
  "key_prefix": "rc_live_ab12",
  "raw_key": "rc_live_ab1234567890abcdef1234567890abcdef1234567890abcdef",
  "expires_at": "2026-12-31T23:59:59Z",
  "created_at": "2026-08-28T14:20:00Z"
}
```

---

## 3. Revoke / Delete API Key

`DELETE /api/v1/api-keys/:id`

Permanently deactivates an API key. Any subsequent requests using this key will immediately receive `401 Unauthorized`.

### Success Response (`204 No Content`):
Empty response body.
