# Common Architectural Mistakes & Antipatterns

Avoid these common mistakes when designing systems around RelayCore:

---

## ❌ Mistake 1: Parsing Request Bodies before Signature Verification
- **Problem**: In frameworks like Express (`app.use(express.json())`), the body is converted to a JS object, reordering JSON keys and altering byte representation.
- **Fix**: Capture the raw request buffer via `express.json({ verify: (req, res, buf) => { req.rawBody = buf; } })`.

## ❌ Mistake 2: Performing Slow Synchronous Processing in Webhook Handlers
- **Problem**: Executing database updates or third-party HTTP calls directly during the inbound HTTP request causes timeouts (5000ms limit).
- **Fix**: Verify signature, push event to an internal job queue (BullMQ/SQS), and return `200 OK` in $<50\text{ms}$.

## ❌ Mistake 3: Forgetting Idempotency Checks on Receivers
- **Problem**: Network failures cause duplicate webhook dispatches. Executing business logic twice (e.g. charging a card) results in data corruption.
- **Fix**: Check `event_id` against a Redis cache before executing business side effects.

## ❌ Mistake 4: Putting API Keys in Client-Side Frontend Code
- **Problem**: Hardcoding programmatic API keys in React or Vue apps leaks credentials to the public internet.
- **Fix**: Only use API keys on secure backend servers. Authenticate frontend users via JWT login sessions.
