# Webhook Sender Guide

This guide explains how backend applications and microservices can publish events into RelayCore to take advantage of fan-out delivery, retries, payload transformation, and dead-letter queueing.

---

## 📡 Ingestion Entrypoint

`POST /hooks/:slug`

RelayCore assigns a unique URL slug to each Inbound Source (e.g. `customer-billing-events`).

### Required Headers:
- `Content-Type: application/json`
- `X-Event-Type: <domain.action>` (e.g. `order.created`, `invoice.paid`)

---

## 💻 Code Examples

### 1. Python (Requests)
```python
import requests
import json

def publish_event(event_type: str, data: dict):
    url = "http://localhost:3001/hooks/customer-billing-events"
    headers = {
        "Content-Type": "application/json",
        "X-Event-Type": event_type
    }
    payload = {
        "event": event_type,
        "data": data
    }
    response = requests.post(url, json=payload, headers=headers, timeout=5)
    response.raise_for_status()
    print(f"Event dispatched: {response.json()['id']}")
```

### 2. Go (net/http)
```go
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type WebhookEvent struct {
	Event string                 `json:"event"`
	Data  map[string]interface{} `json:"data"`
}

func publishEvent(eventType string, data map[string]interface{}) error {
	payload := WebhookEvent{
		Event: eventType,
		Data:  data,
	}
	bodyBytes, err := json.Marshal(payload)
	if err != nil {
		return err
	}

	req, err := http.NewRequest("POST", "http://localhost:3001/hooks/customer-billing-events", bytes.NewBuffer(bodyBytes))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Event-Type", eventType)

	client := &http.Client{Timeout: 5 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusAccepted {
		return fmt.Errorf("unexpected status: %d", resp.StatusCode)
	}
	return nil
}
```
