# Security Model & Threat Mitigation

RelayCore is engineered with defense-in-depth security principles to protect multi-tenant data isolation, cryptographic integrity, and sensitive webhook payloads.

---

## 🛡️ Core Security Architecture

### 1. Multi-Tenant Database Segregation
- Every resource table (`sources`, `destinations`, `subscriptions`, `events`, `deliveries`, `api_keys`, `dlq`) contains a `tenant_id UUID` column.
- Authentication middleware validates the JWT token or API key and binds an `AuthenticatedTenant` context to every Axum request.
- All SQL queries strictly filter by `WHERE tenant_id = $1`. Cross-tenant querying is impossible.

### 2. Cryptographic Constant-Time Signature Validation
- Webhook signature comparison uses `subtle::ConstantTimeEq` to prevent timing attacks.
- Standard string equality (`==`) leaks timing information about how many leading bytes matched; constant-time comparison evaluates all bytes in identical CPU cycle counts.

### 3. Signing Secret Encryption at Rest
- HMAC signing secrets and webhook secrets are encrypted at rest in PostgreSQL using AES-256-GCM authenticated encryption (`ENCRYPTION_KEY`).
- Even with direct read access to database dumps, secrets cannot be decrypted without the host master encryption key.

### 4. Timestamp Tolerance & Replay Attack Defense
- Inbound webhooks with timestamped headers (e.g. Stripe `t=...`, Shopify) are checked against the server's clock.
- Webhooks with timestamps older than `tolerance_seconds` (default: 300 seconds) are rejected immediately to prevent malicious replay of intercepted historical webhooks.

### 5. Sensitive Payload Access Protection
- In the dashboard, raw JSON payloads are never expanded or rendered automatically.
- Users must explicitly click "View Raw Payload", reducing accidental exposure during screen shares.
- Plaintext API keys and signing secrets are shown **only once** upon creation.
