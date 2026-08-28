# Glossary of Terms

- **Inbound Source**: A configured entrypoint with a unique URL slug (`/hooks/:slug`) that ingests external webhooks.
- **Target Destination**: A customer-registered downstream server endpoint where webhooks are delivered.
- **Routing Subscription**: A declarative routing rule linking an Inbound Source to a Target Destination with event type wildcard filters.
- **Event**: An immutable snapshot of an ingested webhook payload and metadata.
- **Delivery**: An execution task created to deliver an Event to a specific Destination.
- **Delivery Attempt**: An individual HTTP POST request made by a background worker for a delivery, logging HTTP status, latency, and response bodies.
- **Dead Letter Queue (DLQ)**: Quarantine storage for deliveries that have exhausted all retry attempts.
- **Circuit Breaker**: An automated fault tolerance mechanism that trips open when a destination encounters repeated consecutive failures.
- **Exponential Backoff**: A retry delay formula ($2^n \times \text{base} + \text{jitter}$) that increases wait intervals between subsequent failure retries.
- **Constant-Time Comparison**: A cryptographic technique to compare hashes in fixed CPU clock cycles, preventing side-channel timing attacks.
- **Keyset Cursor Pagination**: Database pagination using indexed values and opaque base64 tokens rather than slow SQL `OFFSET`.
