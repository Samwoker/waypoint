# Node.js Integration Guide

This guide demonstrates how to integrate modern **Node.js (v18+, v20+, v22+)** applications with RelayCore using standard native `fetch` (or `undici`/`axios`) without third-party bloat.

---

## 📦 Setting Up an API Client

Create a modular, reusable API client (`relaycore-client.ts`):

```typescript
export interface RelayCoreClientConfig {
  baseUrl: string;
  apiKey: string;
}

export class RelayCoreClient {
  private baseUrl: string;
  private apiKey: string;

  constructor(config: RelayCoreClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/+$/, '');
    this.apiKey = config.apiKey;
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.apiKey}`,
        ...(options.headers || {}),
      },
    });

    if (!response.ok) {
      const errorText = await response.text();
      let message = `RelayCore HTTP ${response.status}: ${response.statusText}`;
      try {
        const errorJson = JSON.parse(errorText);
        message = errorJson.message || errorJson.error || message;
      } catch (_) {}
      throw new Error(message);
    }

    if (response.status === 204) return {} as T;
    return response.json();
  }

  // --- Webhook Ingestion ---
  async sendWebhook(slug: string, payload: any, eventType?: string): Promise<{ id: string; status: string }> {
    const response = await fetch(`${this.baseUrl}/hooks/${slug}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(eventType ? { 'X-Event-Type': eventType } : {}),
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      throw new Error(`Failed to send webhook: HTTP ${response.status}`);
    }

    return response.json();
  }

  // --- Source Management ---
  async createSource(name: string, slug: string, verificationType: string = 'none') {
    return this.request('/api/v1/sources', {
      method: 'POST',
      body: JSON.stringify({ name, slug, verification_type: verificationType }),
    });
  }

  // --- Destination Management ---
  async createDestination(name: string, url: string, timeoutMs: number = 5000) {
    return this.request('/api/v1/destinations', {
      method: 'POST',
      body: JSON.stringify({ name, url, timeout_ms: timeoutMs }),
    });
  }

  // --- Delivery Replay ---
  async replayDelivery(deliveryId: string) {
    return this.request(`/api/v1/deliveries/${deliveryId}/replay`, { method: 'POST' });
  }
}
```

---

## 🚀 Publishing Events in Node.js

```typescript
import { RelayCoreClient } from './relaycore-client';

const relay = new RelayCoreClient({
  baseUrl: process.env.RELAYCORE_URL || 'http://localhost:3001',
  apiKey: process.env.RELAYCORE_API_KEY || 'rc_live_ab123456...',
});

async function publishUserRegisteredEvent(user: { id: string; email: string; plan: string }) {
  try {
    const result = await relay.sendWebhook('customer-events', {
      event: 'user.registered',
      data: {
        userId: user.id,
        email: user.email,
        plan: user.plan,
        registeredAt: new Date().toISOString(),
      },
    }, 'user.registered');

    console.log(`Event queued in RelayCore with ID: ${result.id}`);
  } catch (error) {
    console.error('Failed to publish webhook event:', error);
  }
}
```

---

## ⏭️ Next Steps

- Build a production [Express.js Webhook Receiver](./expressjs.md).
- Learn about [Receiver Best Practices & Idempotency](./receiver-guide.md).
