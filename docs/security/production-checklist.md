# Production Hardening Checklist

Before deploying RelayCore to production, verify every item on this checklist:

---

## 🔒 Security Checklist

- [ ] **Strong JWT Secret Configured**: `JWT_SECRET` is set to a cryptographically random string (minimum 32 characters / 256 bits).
- [ ] **Master Encryption Key Configured**: `ENCRYPTION_KEY` is set to a secure 32-byte key for AES-256 secret encryption.
- [ ] **HTTPS Enforced Everywhere**: Reverse proxy / Load Balancer terminates TLS 1.3 for all public traffic.
- [ ] **HTTPS Enforced for Destinations**: `ENFORCE_HTTPS_IN_PRODUCTION=true` is enabled to prevent delivering webhooks over unencrypted HTTP.
- [ ] **Database & Redis Credentials Secured**: Never commit database passwords or Redis connection strings to Git. Use environment secrets or AWS Secrets Manager / HashiCorp Vault.
- [ ] **CORS Origins Restricted**: Set `CORS_ALLOWED_ORIGINS` to your production frontend domain (e.g. `https://app.relaycore.dev`) instead of `*`.
- [ ] **Database Network Isolation**: PostgreSQL and Redis instances are deployed in private subnets with access restricted to API and Worker security groups.
- [ ] **Prometheus Metrics Secured**: Ensure `/metrics` is accessible only to internal Prometheus scrapers and not exposed publicly.
- [ ] **Automated DLQ Alerting**: Configure alerts when `relay_dead_letter_count > 0` to immediately investigate broken downstream receivers.
