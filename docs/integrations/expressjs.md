# Express.js Webhook Receiver Integration

This guide provides a production-ready **Express.js** webhook receiver demonstrating:
1. Capturing the raw unparsed request body for cryptographic signature verification.
2. Constant-time HMAC-SHA256 signature verification.
3. Fast `200 OK` acknowledgment before asynchronous background processing.
4. Idempotency protection to handle retried webhooks safely.

---

## ⚠️ Common Express.js Pitfall: `express.json()`

> **CRITICAL**: Standard `app.use(express.json())` parses request streams into JavaScript objects and destroys raw byte formatting. Cryptographic HMAC verification will **FAIL** if computed on re-serialized JSON. You must capture the raw buffer using `verify` callback options or `express.raw()`.

---

## 🛠️ Complete Production Express Server

```typescript
import express, { Request, Response } from 'express';
import crypto from 'crypto';

const app = express();
const PORT = process.env.PORT || 4000;
const WEBHOOK_SIGNING_SECRET = process.env.WEBHOOK_SECRET || 'whsec_9nbTh0nKCR4fwPesGPm/e8NS14hRtlLx0smBpdgNpW8=';

// 1. Preserve Raw Buffer alongside JSON parsing
app.use(
  express.json({
    verify: (req: any, _res, buf) => {
      req.rawBody = buf;
    },
  })
);

// 2. Cryptographic Signature Verification Helper
function verifyRelayCoreSignature(
  rawBody: Buffer,
  signatureHeader: string | undefined,
  secret: string,
  toleranceSeconds: number = 300
): boolean {
  if (!signatureHeader || !rawBody) return false;

  try {
    // Expected format: t=1614555845,v1=5257a869e7ecebeda32affa62cdca3fa51cad7e77a0e56ff536d0ce8e108d8bd
    const parts = signatureHeader.split(',');
    const timestampPart = parts.find((p) => p.startsWith('t='));
    const sigPart = parts.find((p) => p.startsWith('v1='));

    if (!timestampPart || !sigPart) return false;

    const timestamp = parseInt(timestampPart.substring(2), 10);
    const signature = sigPart.substring(3);

    // Enforce timestamp tolerance defense against replay attacks
    const now = Math.floor(Date.now() / 1000);
    if (Math.abs(now - timestamp) > toleranceSeconds) {
      console.warn(`[Webhook Security] Timestamp expired: ${timestamp} vs ${now}`);
      return false;
    }

    // Compute expected HMAC-SHA256
    const signedPayload = `${timestamp}.${rawBody.toString('utf8')}`;
    const expectedSig = crypto
      .createHmac('sha256', secret)
      .update(signedPayload, 'utf8')
      .digest('hex');

    // Constant-time comparison to prevent timing attacks
    return crypto.timingSafeEqual(
      Buffer.from(signature, 'hex'),
      Buffer.from(expectedSig, 'hex')
    );
  } catch (err) {
    console.error('[Webhook Security] Verification error:', err);
    return false;
  }
}

// In-memory idempotency cache (Use Redis in production clustering)
const processedEvents = new Set<string>();

// 3. Webhook Receiver Route
app.post('/api/webhooks/relaycore', async (req: any, res: Response) => {
  const sigHeader = req.headers['x-signature'] || req.headers['stripe-signature'];

  // Step 1: Verify Signature
  const isValid = verifyRelayCoreSignature(req.rawBody, sigHeader as string, WEBHOOK_SIGNING_SECRET);
  if (!isValid) {
    console.error('[Webhook] Signature verification failed');
    return res.status(401).json({ error: 'Invalid webhook signature' });
  }

  const event = req.body;
  const eventId = event.id || req.headers['x-event-id'];

  // Step 2: Idempotency Check
  if (eventId && processedEvents.has(eventId)) {
    console.log(`[Webhook] Duplicate event ${eventId} received, returning fast 200`);
    return res.status(200).json({ received: true, duplicate: true });
  }

  // Step 3: Fast Acknowledgment
  // Respond 200 OK immediately so RelayCore knows the webhook was delivered
  res.status(200).json({ received: true });

  // Step 4: Asynchronous Processing
  if (eventId) processedEvents.add(eventId);

  setImmediate(async () => {
    try {
      console.log(`[Worker] Processing event: ${event.event || event.type}`);
      // Execute your database business logic here (e.g. fulfill order, credit account)
    } catch (processError) {
      console.error(`[Worker] Failed to process event ${eventId}:`, processError);
    }
  });
});

app.listen(PORT, () => {
  console.log(`Webhook receiver listening on port ${PORT}`);
});
```

---

## ⏭️ Next Steps

- Review [Webhook Receiver Best Practices](./receiver-guide.md).
- Explore [Webhook Sender Guide](./sender-guide.md).
