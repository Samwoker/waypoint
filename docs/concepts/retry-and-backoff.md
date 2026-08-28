# Retries & Exponential Backoff

When a downstream destination server encounters temporary failures, network interruptions, 5xx internal server errors, or gateway timeouts, RelayCore prevents data loss through **Automated Exponential Backoff with Jitter**.

---

## 📐 The Retry Algorithm

RelayCore uses an exponential backoff formula with randomized full jitter to prevent the "thundering herd" problem on recovering downstream services:

$$\text{Delay}(n) = \min\left(\text{MaxDelay},\, \text{BaseDelay} \times 2^n\right) + \text{RandomJitter}$$

### Parameters:
- **$n$**: The current attempt count ($0, 1, 2, \dots$).
- **$\text{BaseDelay}$**: Initial retry backoff base (Default: `10 seconds`).
- **$\text{MaxDelay}$**: Cap on maximum wait interval (Default: `3600 seconds` / 1 hour).
- **$\text{RandomJitter}$**: Uniform random interval $[0, 0.2 \times \text{Delay}]$ to distribute concurrent retries across time.

---

## 📊 Example Retry Progression

Assuming `max_retries = 5`, `BaseDelay = 10s`:

| Attempt | Failure Cause | Calculated Delay | Approximate Execution Time | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Initial Dispatch** | 503 Service Unavailable | Immediate | $T + 0\text{s}$ | `retry` |
| **Retry 1** | 504 Gateway Timeout | $\approx 10\text{s} + \text{jitter}$ | $T + 11\text{s}$ | `retry` |
| **Retry 2** | Connection Refused | $\approx 20\text{s} + \text{jitter}$ | $T + 33\text{s}$ | `retry` |
| **Retry 3** | 500 Internal Server Error | $\approx 40\text{s} + \text{jitter}$ | $T + 75\text{s}$ | `retry` |
| **Retry 4** | 500 Internal Server Error | $\approx 80\text{s} + \text{jitter}$ | $T + 158\text{s}$ | `retry` |
| **Retry 5** | 500 Internal Server Error | — | $T + 322\text{s}$ | `dead_letter` (DLQ) |

---

## 🚦 Retryable vs. Terminal HTTP Status Codes

RelayCore classifies HTTP responses to determine whether a delivery should be retried or immediately marked as permanently failed:

### Retryable Responses (Triggers Backoff Loop):
- **Network / Socket Errors**: Connection refused, DNS resolution failure, TCP reset.
- **HTTP Client Timeouts**: Execution exceeds `timeout_ms` (e.g. 5000ms).
- **HTTP 408**: Request Timeout.
- **HTTP 429**: Too Many Requests (Rate Limited).
- **HTTP 500**: Internal Server Error.
- **HTTP 502**: Bad Gateway.
- **HTTP 503**: Service Unavailable.
- **HTTP 504**: Gateway Timeout.

### Terminal Success Responses (Marks Delivered):
- **HTTP 200 OK**, **201 Created**, **202 Accepted**, **204 No Content**.

---

## 📦 What Happens When Retries Are Exhausted?

When a delivery reaches its configured `max_retries` threshold:
1. The delivery status changes from `retry` $\to$ `dead_letter`.
2. The delivery is quarantined into the **Dead Letter Queue (DLQ)**.
3. The destination's consecutive failure counter is incremented (which may trigger the destination's circuit breaker).
4. The underlying webhook `Event` remains intact in PostgreSQL for later manual or automated replay.

---

## ⏭️ Next Steps

- Learn about [Circuit Breakers & Fault Tolerance](./circuit-breaker.md).
- Understand how to recover failed webhooks in the [Dead Letter Queue (DLQ)](../api/dlq.md).
- View the [Deliveries API Reference](../api/deliveries.md).
