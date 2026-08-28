# Frequently Asked Questions (FAQ)

### Q: Does RelayCore store sensitive payload data?
**A:** RelayCore stores webhook payloads in PostgreSQL to enable debugging, delivery attempts, and replays. Sensitive payload inspection in the UI is protected behind explicit user clicks and can be audited. HMAC signing secrets are encrypted at rest using AES-256-GCM.

### Q: How does RelayCore handle high-throughput traffic spikes?
**A:** Inbound HTTP requests return `202 Accepted` in `<5ms` by immediately persisting the event and pushing delivery jobs to Redis. Background workers process deliveries asynchronously, buffering traffic spikes without overwhelming downstream destinations.

### Q: Can I replay webhooks if my receiving server was down for hours?
**A:** Yes. Any delivery that failed and entered the Dead Letter Queue (DLQ) can be replayed with one click in the UI or via `POST /api/v1/dlq/retry-all`.

### Q: Does RelayCore support wildcard subscriptions?
**A:** Yes. You can route all events using `*`, or use prefix wildcard patterns such as `payment.*` to match `payment.created`, `payment.succeeded`, and `payment.failed`.

### Q: How do I rotate an Inbound Source signing secret without downtime?
**A:** Call `POST /api/v1/sources/:id/rotate-secret`. RelayCore immediately generates and encrypts a new secret while auditing the rotation. Update your upstream provider's console with the newly revealed secret.
