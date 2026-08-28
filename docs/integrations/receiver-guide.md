# Webhook Receiver Best Practices

Building reliable webhook receivers requires following key architectural patterns to avoid dropped events, cascading timeouts, and duplicate executions.

---

## 🏆 The 4 Golden Rules of Webhook Receivers

### 1. Respond Immediately with `200 OK`
- Do **NOT** perform long-running tasks (e.g. video encoding, PDF generation, sync calls to third-party CRMs) during the inbound HTTP request.
- RelayCore workers enforce strict timeouts (typically `5000ms`). If your server takes 6000ms to respond, RelayCore marks the delivery as failed and initiates a retry loop.
- **Best Practice**: Verify the signature, push the event to your local queue (e.g. BullMQ, Celery, SQS), and return `200 OK` in `<50ms`.

### 2. Implement Idempotency Checks
- In distributed systems, network retries mean your receiver **WILL** occasionally receive the same webhook event more than once (At-Least-Once Delivery).
- **Best Practice**: Store processed `event_id` keys in Redis with a 24-hour TTL. If an `event_id` is already marked as processed, return `200 OK` immediately without executing duplicate side effects (like charging a credit card twice).

### 3. Verify Signatures on Raw Bytes
- Always verify HMAC signatures using the original unparsed request stream.
- Re-serialized JSON objects have different whitespace, key order, and float formatting, causing cryptographic digest verification to fail.

### 4. Return Appropriate HTTP Status Codes
- **Return 2xx (200, 201, 202, 204)**: Webhook successfully received and queued.
- **Return 4xx / 5xx**: If your database is down or your service is temporarily overloaded, return 500/503. RelayCore will catch this status code and automatically back off and retry later.
