# 5-Minute Quickstart Tutorial

In this tutorial, you will:
1. Register a tenant workspace and obtain an authentication token.
2. Create an **Inbound Source** with a dedicated `/hooks/:slug` entrypoint.
3. Register a **Target Destination** (downstream server).
4. Create a **Routing Subscription** linking the source to the destination.
5. Ingest a real webhook event and trace the outbound delivery attempt.

---

## Prerequisites

Ensure the RelayCore API is running locally:
```bash
curl -s http://localhost:3001/healthz
# Should return: {"db":"ok","queue":"ok"}
```

---

## Step 1: Register Tenant & Get Auth Token

Create a new tenant organization and retrieve your JWT Bearer token:

```bash
REGISTER_RES=$(curl -s -X POST http://localhost:3001/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "developer@example.com",
    "password": "SecurePassword123!",
    "tenant_name": "Acme Payments"
  }')

echo "$REGISTER_RES"
```

Save the `access_token` into an environment variable:
```bash
export TOKEN=$(echo "$REGISTER_RES" | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)
echo "Your Auth Token: $TOKEN"
```

---

## Step 2: Create an Inbound Source

Create an Inbound Source with the slug `stripe-inbound`:

```bash
SOURCE_RES=$(curl -s -X POST http://localhost:3001/api/v1/sources \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Stripe Production",
    "slug": "stripe-inbound",
    "provider": "stripe",
    "verification_type": "none"
  }')

echo "$SOURCE_RES"
```

Extract the `id` of the created source:
```bash
export SOURCE_ID=$(echo "$SOURCE_RES" | grep -o '"id":"[^"]*' | cut -d'"' -f4)
echo "Source ID: $SOURCE_ID"
```

Your public ingestion endpoint is now live at:
`http://localhost:3001/hooks/stripe-inbound`

---

## Step 3: Register a Target Destination

Register an external endpoint (e.g. `https://httpbin.org/post` or your local server) where RelayCore should deliver events:

```bash
DEST_RES=$(curl -s -X POST http://localhost:3001/api/v1/destinations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Internal Billing API",
    "url": "https://httpbin.org/post",
    "timeout_ms": 5000,
    "max_retries": 3,
    "rate_limit_rps": 100
  }')

echo "$DEST_RES"
```

Extract the `id` of the created destination:
```bash
export DEST_ID=$(echo "$DEST_RES" | grep -o '"id":"[^"]*' | cut -d'"' -f4)
echo "Destination ID: $DEST_ID"
```

---

## Step 4: Create a Routing Subscription

Connect your Inbound Source to your Target Destination and filter for `payment.*` event types:

```bash
SUB_RES=$(curl -s -X POST http://localhost:3001/api/v1/subscriptions \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "source_id": "'$SOURCE_ID'",
    "destination_id": "'$DEST_ID'",
    "event_types": ["payment.created", "payment.succeeded", "charge.refunded"]
  }')

echo "$SUB_RES"
```

---

## Step 5: Ingest a Live Webhook

Send a simulated webhook event into RelayCore's public ingestion entrypoint:

```bash
HOOK_RES=$(curl -s -X POST http://localhost:3001/hooks/stripe-inbound \
  -H "Content-Type: application/json" \
  -H "X-Event-Type: payment.succeeded" \
  -d '{
    "event": "payment.succeeded",
    "data": {
      "id": "ch_3MjjkwLkdIwHu7ix",
      "amount": 4999,
      "currency": "usd",
      "customer": "cus_99881122"
    }
  }')

echo "$HOOK_RES"
```

Expected output:
```json
{
  "id": "d2344500-3731-4c8d-9763-9486789e31cb",
  "event_type": "payment.succeeded",
  "status": "received",
  "created_at": "2026-08-28T12:30:00.000Z"
}
```

---

## Step 6: Inspect Ingested Event & Delivery Dispatches

Query the event detail and its fan-out dispatches:

```bash
export EVENT_ID=$(echo "$HOOK_RES" | grep -o '"id":"[^"]*' | cut -d'"' -f4)

# 1. Fetch Event Detail
curl -s -X GET "http://localhost:3001/api/v1/events/$EVENT_ID" \
  -H "Authorization: Bearer $TOKEN"

# 2. Fetch Deliveries generated for this event
curl -s -X GET "http://localhost:3001/api/v1/events/$EVENT_ID/deliveries" \
  -H "Authorization: Bearer $TOKEN"
```

---

## 🎉 Congratulations!

You have successfully ingested a webhook, routed it across a subscription rule, and verified its delivery trace!

### What's Next?
- Read the [Core Concepts Guide](../concepts/core-concepts.md).
- Learn how to verify [HMAC Signatures in Express.js](../integrations/expressjs.md).
- Explore the [API Reference](../api/overview.md).
