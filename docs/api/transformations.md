# Transformation Engine API

The Transformation Engine allows developers to reshape and map incoming webhook JSON payloads dynamically using JSONPath templates before dispatching them to downstream destinations.

---

## 1. Test Transformation in Sandbox

`POST /api/v1/transformations/test`

Executes a dry-run evaluation of a JSONPath template against a provided sample payload without modifying any database records.

### Request Body:
```json
{
  "template": "{\"order_id\": \"$.data.id\", \"total_amount\": \"$.data.amount\", \"customer_email\": \"$.customer.email\"}",
  "payload": {
    "data": {
      "id": "ord_998877",
      "amount": 149.99
    },
    "customer": {
      "email": "sarah@example.com"
    }
  }
}
```

### Success Response (`200 OK`):
```json
{
  "transformed_payload": {
    "order_id": "ord_998877",
    "total_amount": 149.99,
    "customer_email": "sarah@example.com"
  }
}
```

---

## 2. List Configured Transformations

`GET /api/v1/transformations`

Retrieves all active transformations associated with tenant subscriptions.

---

## 3. Create Subscription Transformation

`POST /api/v1/transformations`

Attaches a JSONPath transformation rule to an existing subscription.

### Request Body:
```json
{
  "subscription_id": "0c48982d-e741-4ba2-b41b-be4ffe97d02f",
  "template": "{\"external_id\": \"$.id\", \"status\": \"$.data.status\"}"
}
```

### Success Response (`201 Created`):
```json
{
  "id": "1b2c3d4e-5f6a-7b8c-9d0e-1f2a3b4c5d6e",
  "subscription_id": "0c48982d-e741-4ba2-b41b-be4ffe97d02f",
  "template": "{\"external_id\": \"$.id\", \"status\": \"$.data.status\"}",
  "created_at": "2026-08-28T14:45:00Z"
}
```
