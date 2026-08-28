# Inbound Sources API

Inbound Sources define webhook ingress entrypoints, provider types, and cryptographic HMAC verification settings.

---

## 1. List Inbound Sources

`GET /api/v1/sources`

Retrieves all configured inbound webhook sources for the tenant.

### Headers:
`Authorization: Bearer <TOKEN>`

### Success Response (`200 OK`):
```json
[
  {
    "id": "2e38d7c0-ebaf-4441-b9da-5c35ff6e4a50",
    "tenant_id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
    "name": "Stripe Live",
    "slug": "stripe-live",
    "provider": "stripe",
    "verification_type": "stripe",
    "is_active": true,
    "has_secret": true,
    "tolerance_seconds": 300,
    "created_at": "2026-08-28T12:00:00Z",
    "updated_at": "2026-08-28T12:00:00Z"
  }
]
```

---

## 2. Create Inbound Source

`POST /api/v1/sources`

Creates an inbound webhook source and returns the generated cryptographic signing secret.

### Verification Types Supported:
- `generic_hmac`: Standard Hex HMAC-SHA256 evaluated against `X-Signature` header.
- `stripe`: Stripe v1 timestamped HMAC signature (`Stripe-Signature: t=...,v1=...`).
- `github`: GitHub SHA-256 HMAC (`X-Hub-Signature-256: sha256=...`).
- `shopify`: Shopify Base64 HMAC (`X-Shopify-Hmac-Sha256: ...`).
- `none`: Open endpoint without cryptographic signature enforcement.

### Request Body:
```json
{
  "name": "Stripe Production",
  "slug": "stripe-prod",
  "provider": "stripe",
  "verification_type": "stripe",
  "tolerance_seconds": 300
}
```

### Success Response (`201 Created`):
```json
{
  "id": "2e38d7c0-ebaf-4441-b9da-5c35ff6e4a50",
  "tenant_id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
  "name": "Stripe Production",
  "slug": "stripe-prod",
  "provider": "stripe",
  "verification_type": "stripe",
  "is_active": true,
  "has_secret": true,
  "secret": "whsec_9nbTh0nKCR4fwPesGPm/e8NS14hRtlLx0smBpdgNpW8=",
  "tolerance_seconds": 300,
  "created_at": "2026-08-28T12:00:00Z",
  "updated_at": "2026-08-28T12:00:00Z"
}
```

---

## 3. Get Source Details

`GET /api/v1/sources/:id`

Retrieves metadata for a specific source.

---

## 4. Rotate Signing Secret

`POST /api/v1/sources/:id/rotate-secret`

Generates a new cryptographic HMAC signing secret for the source, encrypts it at rest, and returns the new plaintext secret once.

### Success Response (`200 OK`):
```json
{
  "secret": "whsec_newGeneratedSecretString1234567890=",
  "rotated_at": "2026-08-28T14:30:00Z"
}
```

---

## 5. Get Source Verification Audit Logs

`GET /api/v1/sources/:id/verification-log?limit=20`

Returns the most recent signature verification audit logs for the source, detailing verified signatures, timestamp mismatches, and validation errors.

### Success Response (`200 OK`):
```json
[
  {
    "id": "8f1a2b3c-4d5e-6f7a-8b9c-0d1e2f3a4b5c",
    "source_id": "2e38d7c0-ebaf-4441-b9da-5c35ff6e4a50",
    "verified": false,
    "error_reason": "Timestamp expired: received timestamp outside 300s tolerance window",
    "received_at": "2026-08-28T14:28:00Z"
  }
]
```

---

## 6. Delete Inbound Source

`DELETE /api/v1/sources/:id`

Deletes the source endpoint. Any subsequent webhooks sent to `/hooks/:slug` will be rejected with `404 Not Found`.

### Success Response (`204 No Content`):
Empty response body.
