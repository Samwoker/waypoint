# Authentication & User API

RelayCore uses Argon2id password hashing and signed JWT Bearer access tokens with refresh token rotation.

---

## 1. Register New Tenant & User

`POST /api/v1/auth/register`

Provisions a new tenant workspace in PostgreSQL, hashes the user password with Argon2id, assigns the `owner` role, and generates an initial JWT access/refresh token pair.

### Request Body:
```json
{
  "email": "lead@company.com",
  "password": "StrongPassword123!",
  "tenant_name": "Acme Payments"
}
```

### Success Response (`200 OK`):
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

---

## 2. User Login

`POST /api/v1/auth/login`

Authenticates existing user credentials and returns signed JWT access and refresh tokens.

### Request Body:
```json
{
  "email": "lead@company.com",
  "password": "StrongPassword123!"
}
```

### Success Response (`200 OK`):
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

### Error Responses:
- `401 Unauthorized`: Invalid email or password.
- `400 Bad Request`: Missing required email or password fields.

---

## 3. Refresh Access Token

`POST /api/v1/auth/refresh`

Exchanges a valid refresh token for a new short-lived JWT access token without requiring user password re-entry.

### Request Body:
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### Success Response (`200 OK`):
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

---

## 4. Get Current User & Tenant Profile

`GET /api/v1/auth/me`

Retrieves the currently authenticated user's session claims, roles, and tenant metadata.

### Headers:
`Authorization: Bearer <TOKEN>`

### Success Response (`200 OK`):
```json
{
  "tenant_id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
  "role": "owner",
  "is_admin": true,
  "scope": "full",
  "tenant": {
    "id": "e11f57b6-e337-4943-a8c6-60a0c42768af",
    "name": "Acme Payments",
    "slug": "acme-payments-8899",
    "created_at": "2026-08-28T12:00:00.000Z",
    "updated_at": "2026-08-28T12:00:00.000Z"
  }
}
```
