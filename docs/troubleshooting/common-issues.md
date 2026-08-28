# Troubleshooting Common Issues

This guide provides solutions to common operational and configuration issues.

---

## 1. Database Connection Refused

- **Symptom**: API crashes on startup with `error: ConnectionRefused` or `failed to connect to postgres`.
- **Diagnosis**: Verify PostgreSQL is listening on the expected port:
  ```bash
  pg_isready -h localhost -p 5432
  ```
- **Solution**: Ensure `DATABASE_URL` in `.env` contains valid credentials and that the `webhook_relay` database exists. Apply migrations using `sqlx database setup` or `psql`.

---

## 2. Webhook Signature Verification Mismatch (HTTP 401)

- **Symptom**: Webhook ingestion fails with `401 Unauthorized: Invalid webhook signature`.
- **Causes**:
  1. Secret mismatch between provider console and RelayCore source.
  2. Timestamp skew: The sending server clock differs by $> 300\text{s}$.
  3. Header name mismatch: Ensure Stripe uses `Stripe-Signature`, GitHub uses `X-Hub-Signature-256`, etc.
- **Solution**: Check the Source Verification Log (`GET /api/v1/sources/:id/verification-log`) to inspect the exact failure reason recorded by the cryptographic engine.

---

## 3. Deliveries Stuck in `pending` (Not Dispatching)

- **Symptom**: Ingested events show in the dashboard, but deliveries remain in `pending` status.
- **Causes**:
  1. Background worker process is not running.
  2. Redis connection failed or Redis stream is blocked.
- **Solution**: Start the background worker process (`cargo run -p worker`) and verify Redis connectivity (`redis-cli ping`).
