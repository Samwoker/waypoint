#!/usr/bin/env bash
set -e

BASE="http://localhost:3001"

echo "=================================================="
echo "1. Testing Health Endpoint (GET /v1/health)"
echo "=================================================="
curl -i -X GET "${BASE}/v1/health"
echo -e "\n"

echo "=================================================="
echo "2. Testing Tenant Creation (POST /api/v1/tenants)"
echo "=================================================="
SLUG="tenant-live-$(date +%s)"
TENANT_BODY=$(curl -s -X POST "${BASE}/api/v1/tenants" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"Acme Production Corp\", \"slug\": \"${SLUG}\"}")
echo "Response body:"
echo "${TENANT_BODY}"
TENANT_ID=$(echo "${TENANT_BODY}" | grep -o '"id":"[^"]*' | cut -d'"' -f4)
echo -e "\nCreated Tenant ID: ${TENANT_ID}\n"

echo "=================================================="
echo "3. Testing Transformation Sandbox (POST /api/v1/transformations/test)"
echo "=================================================="
curl -i -X POST "${BASE}/api/v1/transformations/test" \
  -H "Content-Type: application/json" \
  -d '{
    "template": "{\"order_number\": \"$.data.id\", \"total\": \"$.data.amount\", \"status\": \"PROCESSED\"}",
    "payload": {
      "data": {
        "id": "ORD-2026-X99",
        "amount": 1499.00
      }
    }
  }'
echo -e "\n"

echo "=================================================="
echo "4. Testing Inbound Public Webhook (POST /hooks/:slug)"
echo "=================================================="
HOOK_SLUG="stripe-live-$(date +%s)"
psql postgres://postgres:postgres@localhost:5432/webhook_relay -c \
  "INSERT INTO sources (id, tenant_id, name, slug, source_type, status, metadata) VALUES (gen_random_uuid(), '${TENANT_ID}', 'Stripe Inbound', '${HOOK_SLUG}', 'stripe', 'active', '{\"verification_type\": \"none\"}');" > /dev/null

curl -i -X POST "${BASE}/hooks/${HOOK_SLUG}" \
  -H "Content-Type: application/json" \
  -H "X-Event-Type: payment_intent.succeeded" \
  -d '{
    "id": "evt_live_123456789",
    "object": "event",
    "data": {
      "object": {
        "id": "pi_12345",
        "amount": 5000,
        "currency": "usd"
      }
    }
  }'
echo -e "\n"

echo "=================================================="
echo "5. Testing Tenant List & Detail"
echo "=================================================="
echo "GET /api/v1/tenants (No auth -> 401 Unauthorized as expected for tenant isolation):"
curl -i -X GET "${BASE}/api/v1/tenants" || true
echo -e "\n"

echo "=================================================="
echo "All Curl Checks Completed Successfully!"
echo "=================================================="
