import React, { useState } from 'react';
import {
  Activity,
  AlertCircle,
  BookOpen,
  CheckCircle2,
  Code2,
  Cpu,
  Flame,
  Key,
  Layers,
  Radio,
  RefreshCw,
  Send,
  ShieldCheck,
  Zap,
} from 'lucide-react';
import { CodeBlock } from '../components/common/CodeBlock';

export const DocsPage: React.FC = () => {
  const [activeSection, setActiveSection] = useState<string>('intro');

  const navItems = [
    {
      category: 'GETTING STARTED',
      items: [
        { id: 'intro', label: 'Introduction & Architecture' },
        { id: 'quickstart', label: '5-Minute Quickstart' },
        { id: 'auth-tokens', label: 'Auth & JWT Tokens' },
      ],
    },
    {
      category: 'CORE INGESTION & SECURITY',
      items: [
        { id: 'hooks-ingestion', label: 'POST /hooks/:slug (Ingestion)' },
        { id: 'signatures', label: 'HMAC Signature Verification' },
        { id: 'sources-api', label: 'Inbound Sources API' },
      ],
    },
    {
      category: 'DELIVERY & RELIABILITY',
      items: [
        { id: 'destinations-api', label: 'Destinations & Circuit Breakers' },
        { id: 'subscriptions-api', label: 'Subscriptions & Routing Rules' },
        { id: 'deliveries-api', label: 'Deliveries & Replay API' },
        { id: 'dlq-api', label: 'Dead Letter Queue (DLQ) API' },
      ],
    },
    {
      category: 'TRANSFORMATION & UTILITIES',
      items: [
        { id: 'transformations-api', label: 'Transformation Sandbox API' },
        { id: 'apikeys-api', label: 'API Keys & Tenant Quotas' },
        { id: 'health-metrics', label: 'Healthz, Metrics & Stats' },
      ],
    },
  ];

  return (
    <div className="h-[calc(100vh-64px)] flex overflow-hidden animate-in fade-in duration-150">
      {/* Left Navigation Tree */}
      <aside className="w-72 border-r border-zinc-800 bg-[#0c0c0e] overflow-y-auto p-4 space-y-6 shrink-0">
        <div className="flex items-center space-x-2 px-2 py-1 text-xs font-mono font-bold text-white uppercase tracking-wider">
          <BookOpen className="w-4 h-4 text-emerald-400" />
          <span>Documentation Portal</span>
        </div>

        {navItems.map((group) => (
          <div key={group.category} className="space-y-1.5">
            <div className="px-2 text-[10px] font-mono font-semibold text-zinc-500 tracking-wider">
              {group.category}
            </div>
            <div className="space-y-0.5">
              {group.items.map((item) => (
                <button
                  key={item.id}
                  onClick={() => setActiveSection(item.id)}
                  className={`w-full text-left px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                    activeSection === item.id
                      ? 'bg-zinc-800 text-white font-semibold border border-zinc-700/60 shadow-sm'
                      : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/50'
                  }`}
                >
                  {item.label}
                </button>
              ))}
            </div>
          </div>
        ))}
      </aside>

      {/* Docs Center Content Area */}
      <main className="flex-1 overflow-y-auto p-10 max-w-5xl space-y-12 bg-[#09090b]">
        {/* SECTION: Intro */}
        {activeSection === 'intro' && (
          <div className="space-y-6 animate-in fade-in">
            <div>
              <span className="text-xs font-mono text-emerald-400 font-semibold uppercase">Architecture</span>
              <h1 className="text-3xl font-extrabold text-white mt-1 tracking-tight">Introduction to Waypoint</h1>
              <p className="text-sm text-zinc-400 mt-2 leading-relaxed">
                Waypoint is an enterprise webhook ingestion and resilient fan-out relay gateway designed for high-throughput, mission-critical event infrastructure.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="p-5 rounded-xl bg-[#121215] border border-zinc-800 space-y-2">
                <div className="w-8 h-8 rounded-lg bg-emerald-500/10 text-emerald-400 flex items-center justify-center">
                  <ShieldCheck className="w-4 h-4" />
                </div>
                <h3 className="text-sm font-semibold text-white">Cryptographic Verification</h3>
                <p className="text-xs text-zinc-400">
                  Constant-time HMAC-SHA256 signature verification for Stripe, GitHub, Shopify, and custom providers with timestamp tolerance defense.
                </p>
              </div>

              <div className="p-5 rounded-xl bg-[#121215] border border-zinc-800 space-y-2">
                <div className="w-8 h-8 rounded-lg bg-blue-500/10 text-blue-400 flex items-center justify-center">
                  <Cpu className="w-4 h-4" />
                </div>
                <h3 className="text-sm font-semibold text-white">Automated Circuit Breakers</h3>
                <p className="text-xs text-zinc-400">
                  Monitors consecutive downstream timeouts and error responses, tripping open automatically to avoid cascading failures.
                </p>
              </div>

              <div className="p-5 rounded-xl bg-[#121215] border border-zinc-800 space-y-2">
                <div className="w-8 h-8 rounded-lg bg-violet-500/10 text-violet-400 flex items-center justify-center">
                  <Code2 className="w-4 h-4" />
                </div>
                <h3 className="text-sm font-semibold text-white">JSONPath Transformation Engine</h3>
                <p className="text-xs text-zinc-400">
                  Reshapes and maps inbound webhook payloads dynamically before downstream delivery without custom glue code.
                </p>
              </div>

              <div className="p-5 rounded-xl bg-[#121215] border border-zinc-800 space-y-2">
                <div className="w-8 h-8 rounded-lg bg-amber-500/10 text-amber-400 flex items-center justify-center">
                  <RefreshCw className="w-4 h-4" />
                </div>
                <h3 className="text-sm font-semibold text-white">Dead Letter Queue (DLQ)</h3>
                <p className="text-xs text-zinc-400">
                  Quarantine failed deliveries with complete request/response traces and 1-click bulk replay capabilities.
                </p>
              </div>
            </div>
          </div>
        )}

        {/* SECTION: Quickstart */}
        {activeSection === 'quickstart' && (
          <div className="space-y-6 animate-in fade-in">
            <div>
              <span className="text-xs font-mono text-emerald-400 font-semibold uppercase">Tutorial</span>
              <h1 className="text-3xl font-extrabold text-white mt-1 tracking-tight">5-Minute Quickstart</h1>
              <p className="text-sm text-zinc-400 mt-2 leading-relaxed">
                Connect your upstream providers and start streaming webhooks reliably in minutes.
              </p>
            </div>

            <div className="space-y-8 text-sm text-zinc-300">
              <div className="space-y-2">
                <h3 className="text-base font-semibold text-white flex items-center space-x-2">
                  <span className="w-6 h-6 rounded-full bg-zinc-800 border border-zinc-700 text-xs flex items-center justify-center font-mono">1</span>
                  <span>Create an Inbound Source</span>
                </h3>
                <p className="text-xs text-zinc-400">
                  Register a source slug to generate a public ingestion endpoint (e.g. `/hooks/stripe-prod`).
                </p>
                <CodeBlock
                  title="Create Source Endpoint"
                  snippets={[
                    {
                      language: 'cURL',
                      code: `curl -X POST http://localhost:3001/api/v1/sources \\
  -H "Authorization: Bearer <TOKEN>" \\
  -H "Content-Type: application/json" \\
  -d '{
    "name": "Stripe Production Inbound",
    "slug": "stripe-prod",
    "provider": "stripe",
    "verification_type": "stripe"
  }'`,
                    },
                    {
                      language: 'TypeScript',
                      code: `import axios from 'axios';

const response = await axios.post('http://localhost:3001/api/v1/sources', {
  name: 'Stripe Production Inbound',
  slug: 'stripe-prod',
  provider: 'stripe',
  verification_type: 'stripe',
}, {
  headers: { Authorization: 'Bearer <TOKEN>' }
});

console.log('Created source:', response.data);`,
                    },
                    {
                      language: 'Python',
                      code: `import requests

res = requests.post(
    'http://localhost:3001/api/v1/sources',
    json={
        'name': 'Stripe Production Inbound',
        'slug': 'stripe-prod',
        'provider': 'stripe',
        'verification_type': 'stripe'
    },
    headers={'Authorization': 'Bearer <TOKEN>'}
)
print('Created source:', res.json())`,
                    },
                    {
                      language: 'Rust',
                      code: `use reqwest::Client;
use serde_json::json;

let client = Client::new();
let res = client.post("http://localhost:3001/api/v1/sources")
    .bearer_auth(token)
    .json(&json!({
        "name": "Stripe Production Inbound",
        "slug": "stripe-prod",
        "provider": "stripe",
        "verification_type": "stripe"
    }))
    .send()
    .await?;`,
                    },
                  ]}
                />
              </div>

              <div className="space-y-2">
                <h3 className="text-base font-semibold text-white flex items-center space-x-2">
                  <span className="w-6 h-6 rounded-full bg-zinc-800 border border-zinc-700 text-xs flex items-center justify-center font-mono">2</span>
                  <span>Connect a Destination & Routing Subscription</span>
                </h3>
                <p className="text-xs text-zinc-400">
                  Register your downstream server and subscribe it to receive events matching specific type patterns.
                </p>
                <CodeBlock
                  title="Connect Destination & Subscribe"
                  snippets={[
                    {
                      language: 'cURL',
                      code: `# 1. Register destination
curl -X POST http://localhost:3001/api/v1/destinations \\
  -H "Authorization: Bearer <TOKEN>" \\
  -H "Content-Type: application/json" \\
  -d '{
    "name": "Billing Service Receiver",
    "url": "https://api.example.com/webhooks/billing",
    "max_retry_count": 5,
    "timeout_ms": 5000
  }'

# 2. Subscribe destination to source
curl -X POST http://localhost:3001/api/v1/subscriptions \\
  -H "Authorization: Bearer <TOKEN>" \\
  -H "Content-Type: application/json" \\
  -d '{
    "source_id": "<SOURCE_UUID>",
    "destination_id": "<DESTINATION_UUID>",
    "event_types": ["payment_intent.succeeded", "charge.refunded"]
  }'`,
                    },
                    {
                      language: 'TypeScript',
                      code: `// Register destination
const dest = await axios.post('http://localhost:3001/api/v1/destinations', {
  name: 'Billing Service Receiver',
  url: 'https://api.example.com/webhooks/billing',
  max_retry_count: 5,
  timeout_ms: 5000,
}, { headers: { Authorization: 'Bearer <TOKEN>' } });

// Subscribe destination
await axios.post('http://localhost:3001/api/v1/subscriptions', {
  source_id: sourceId,
  destination_id: dest.data.id,
  event_types: ['payment_intent.succeeded', 'charge.refunded'],
}, { headers: { Authorization: 'Bearer <TOKEN>' } });`,
                    },
                  ]}
                />
              </div>

              <div className="space-y-2">
                <h3 className="text-base font-semibold text-white flex items-center space-x-2">
                  <span className="w-6 h-6 rounded-full bg-zinc-800 border border-zinc-700 text-xs flex items-center justify-center font-mono">3</span>
                  <span>Dispatch Webhook to Inbound URL</span>
                </h3>
                <p className="text-xs text-zinc-400">
                  Send live webhooks to `/hooks/:slug`. Waypoint validates cryptographic signatures, evaluates JSONPath transformations, and dispatches to all subscribed endpoints with automated retries.
                </p>
                <CodeBlock
                  title="Dispatch Inbound Webhook"
                  snippets={[
                    {
                      language: 'cURL',
                      code: `curl -X POST http://localhost:3001/hooks/stripe-prod \\
  -H "Content-Type: application/json" \\
  -H "X-Event-Type: payment_intent.succeeded" \\
  -d '{
    "id": "evt_3MjjkwLkdIwHu7ix0snNq8KG",
    "object": "event",
    "data": {
      "amount": 2999,
      "currency": "usd",
      "status": "succeeded"
    }
  }'`,
                    },
                  ]}
                />
              </div>
            </div>
          </div>
        )}

        {/* SECTION: Auth & JWT Tokens */}
        {activeSection === 'auth-tokens' && (
          <div className="space-y-6 animate-in fade-in">
            <div>
              <span className="text-xs font-mono text-emerald-400 font-semibold uppercase">Security</span>
              <h1 className="text-3xl font-extrabold text-white mt-1 tracking-tight">Authentication & Token Management</h1>
              <p className="text-sm text-zinc-400 mt-2 leading-relaxed">
                Waypoint supports Argon2id password authentication with JWT claims, refresh token rotation, and scoped API keys.
              </p>
            </div>

            {/* Login Endpoint */}
            <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2.5">
                  <span className="px-2 py-0.5 rounded text-xs font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                    POST
                  </span>
                  <span className="font-mono text-sm text-white font-bold">/api/v1/auth/login</span>
                </div>
                <span className="text-xs font-mono text-zinc-500">Public</span>
              </div>
              <p className="text-xs text-zinc-300">
                Authenticate with user credentials to receive a signed JWT access token and refresh token.
              </p>

              <CodeBlock
                title="Login Request & Response"
                snippets={[
                  {
                    language: 'cURL',
                    code: `curl -X POST http://localhost:3001/api/v1/auth/login \\
  -H "Content-Type: application/json" \\
  -d '{
    "email": "dev@waypoint.internal",
    "password": "super-secure-password"
  }'`,
                  },
                  {
                    language: 'JSON',
                    code: `// HTTP 200 OK Response
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "rt_8f1b2c3d4e5f...",
  "token_type": "Bearer",
  "expires_in": 86400
}`,
                  },
                ]}
              />
            </div>

            {/* Refresh Token Endpoint */}
            <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2.5">
                  <span className="px-2 py-0.5 rounded text-xs font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                    POST
                  </span>
                  <span className="font-mono text-sm text-white font-bold">/api/v1/auth/refresh</span>
                </div>
                <span className="text-xs font-mono text-zinc-500">Public</span>
              </div>
              <p className="text-xs text-zinc-300">
                Exchange a valid refresh token for a new short-lived JWT access token without re-authenticating.
              </p>
              <CodeBlock
                title="Token Refresh"
                singleLang="bash"
                singleCode={`curl -X POST http://localhost:3001/api/v1/auth/refresh \\
  -H "Content-Type: application/json" \\
  -d '{"refresh_token": "rt_8f1b2c3d4e5f..."}'`}
              />
            </div>
          </div>
        )}

        {/* SECTION: Public Hooks Ingestion */}
        {activeSection === 'hooks-ingestion' && (
          <div className="space-y-6 animate-in fade-in">
            <div>
              <span className="text-xs font-mono text-emerald-400 font-semibold uppercase">Inbound Gateway</span>
              <h1 className="text-3xl font-extrabold text-white mt-1 tracking-tight">POST /hooks/:slug</h1>
              <p className="text-sm text-zinc-400 mt-2 leading-relaxed">
                Public ingestion endpoint for external webhooks. Validates signatures in constant time and immediately queues the event (`202 Accepted`).
              </p>
            </div>

            <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
              <div className="flex items-center space-x-2.5">
                <span className="px-2 py-0.5 rounded text-xs font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                  POST
                </span>
                <span className="font-mono text-sm text-white font-bold">/hooks/:slug</span>
              </div>

              <div className="space-y-2 text-xs">
                <h4 className="font-mono text-zinc-400 font-semibold uppercase">Path Parameters</h4>
                <div className="p-3 bg-zinc-950 rounded-lg border border-zinc-800 font-mono text-zinc-300">
                  <code>:slug</code> — Unique URL slug assigned to the inbound source (e.g. `stripe-payments`, `github-ci`).
                </div>
              </div>

              <div className="space-y-2 text-xs">
                <h4 className="font-mono text-zinc-400 font-semibold uppercase">Signature Verification Headers</h4>
                <ul className="space-y-1.5 font-mono text-zinc-300 list-disc list-inside">
                  <li><code>Stripe-Signature</code>: Timestamped signature (`t=1614...,v1=hex...`)</li>
                  <li><code>X-Hub-Signature-256</code>: GitHub SHA-256 HMAC (`sha256=hex...`)</li>
                  <li><code>X-Shopify-Hmac-Sha256</code>: Shopify Base64 encoded HMAC</li>
                  <li><code>X-Signature</code>: Generic Hex HMAC-SHA256</li>
                </ul>
              </div>

              <CodeBlock
                title="Ingestion Request & Response"
                snippets={[
                  {
                    language: 'HTTP',
                    code: `POST /hooks/stripe-payments HTTP/1.1
Host: api.waypoint.dev
Content-Type: application/json
X-Event-Type: payment.succeeded
Stripe-Signature: t=1614555845,v1=5257a869e7ecebeda32affa62cdca3fa51cad7e77a0e56ff536d0ce8e108d8bd

{
  "id": "evt_123456789",
  "amount": 4900,
  "currency": "usd"
}`,
                  },
                  {
                    language: 'JSON',
                    code: `// HTTP 202 Accepted Response
{
  "status": "accepted",
  "event_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "source_id": "e4d7a8c9-1f2b-4c3d-8e5a-7f9a0b1c2d3e",
  "queued_at": "2026-08-26T16:30:00Z"
}`,
                  },
                ]}
              />
            </div>
          </div>
        )}

        {/* SECTION: HMAC Signatures */}
        {activeSection === 'signatures' && (
          <div className="space-y-6 animate-in fade-in">
            <div>
              <span className="text-xs font-mono text-emerald-400 font-semibold uppercase">Cryptography</span>
              <h1 className="text-3xl font-extrabold text-white mt-1 tracking-tight">Cryptographic Signature Verification</h1>
              <p className="text-sm text-zinc-400 mt-2 leading-relaxed">
                Waypoint computes cryptographic HMAC digests and verifies incoming payloads against provider signatures using constant-time string comparisons to prevent timing attacks.
              </p>
            </div>

            <CodeBlock
              title="Signature Generation Examples"
              snippets={[
                {
                  language: 'TypeScript',
                  code: `import crypto from 'crypto';

function computeStripeSignature(payload: string, secret: string, timestamp: number): string {
  const signedPayload = \`\${timestamp}.\${payload}\`;
  const signature = crypto
    .createHmac('sha256', secret)
    .update(signedPayload, 'utf8')
    .digest('hex');
  return \`t=\${timestamp},v1=\${signature}\`;
}`,
                },
                {
                  language: 'Python',
                  code: `import hmac
import hashlib
import time

def compute_stripe_signature(payload: str, secret: str, timestamp: int) -> str:
    signed_payload = f"{timestamp}.{payload}".encode('utf-8')
    sig = hmac.new(secret.encode('utf-8'), signed_payload, hashlib.sha256).hexdigest()
    return f"t={timestamp},v1={sig}"`,
                },
                {
                  language: 'Rust',
                  code: `use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

fn compute_hmac_sha256(secret: &[u8], payload: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(payload);
    hex::encode(mac.finalize().into_bytes())
}`,
                },
              ]}
            />
          </div>
        )}

        {/* SECTION: Destinations & Circuit Breakers */}
        {activeSection === 'destinations-api' && (
          <div className="space-y-6 animate-in fade-in">
            <div>
              <span className="text-xs font-mono text-emerald-400 font-semibold uppercase">Downstream Egress</span>
              <h1 className="text-3xl font-extrabold text-white mt-1 tracking-tight">Destinations & Circuit Breakers API</h1>
              <p className="text-sm text-zinc-400 mt-2 leading-relaxed">
                Manage target endpoints, retry budgets, timeout thresholds, and automated circuit breaker state recovery.
              </p>
            </div>

            <div className="space-y-6">
              {/* List Destinations */}
              <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-3">
                <div className="flex items-center space-x-2.5">
                  <span className="px-2 py-0.5 rounded text-xs font-mono font-bold bg-blue-500/10 text-blue-400 border border-blue-500/20">
                    GET
                  </span>
                  <span className="font-mono text-sm text-white font-bold">/api/v1/destinations</span>
                </div>
                <p className="text-xs text-zinc-300">List all registered target destination endpoints for the active tenant.</p>
                <CodeBlock
                  title="List Destinations"
                  singleLang="bash"
                  singleCode={`curl -X GET http://localhost:3001/api/v1/destinations \\
  -H "Authorization: Bearer <TOKEN>"`}
                />
              </div>

              {/* Reset Circuit Breaker */}
              <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-3">
                <div className="flex items-center space-x-2.5">
                  <span className="px-2 py-0.5 rounded text-xs font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                    POST
                  </span>
                  <span className="font-mono text-sm text-white font-bold">/api/v1/destinations/:id/reset-circuit</span>
                </div>
                <p className="text-xs text-zinc-300">
                  Manually reset an open or half-open circuit breaker back to healthy closed state after upstream server recovery.
                </p>
                <CodeBlock
                  title="Reset Circuit"
                  singleLang="bash"
                  singleCode={`curl -X POST http://localhost:3001/api/v1/destinations/<DESTINATION_ID>/reset-circuit \\
  -H "Authorization: Bearer <TOKEN>"`}
                />
              </div>
            </div>
          </div>
        )}

        {/* SECTION: Transformations Sandbox */}
        {activeSection === 'transformations-api' && (
          <div className="space-y-6 animate-in fade-in">
            <div>
              <span className="text-xs font-mono text-emerald-400 font-semibold uppercase">Sandbox</span>
              <h1 className="text-3xl font-extrabold text-white mt-1 tracking-tight">Transformation Sandbox API</h1>
              <p className="text-sm text-zinc-400 mt-2 leading-relaxed">
                Test and preview JSONPath template transformations dynamically before attaching them to live subscriptions.
              </p>
            </div>

            <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
              <div className="flex items-center space-x-2.5">
                <span className="px-2 py-0.5 rounded text-xs font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                  POST
                </span>
                <span className="font-mono text-sm text-white font-bold">/api/v1/transformations/test</span>
              </div>
              <p className="text-xs text-zinc-300">
                Execute a dry-run JSONPath evaluation against a provided sample JSON payload.
              </p>

              <CodeBlock
                title="Test Transformation Request"
                snippets={[
                  {
                    language: 'cURL',
                    code: `curl -X POST http://localhost:3001/api/v1/transformations/test \\
  -H "Authorization: Bearer <TOKEN>" \\
  -H "Content-Type: application/json" \\
  -d '{
    "template": "{\\"order_id\\": \\"$.id\\", \\"total_usd\\": \\"$.data.amount\\"}",
    "payload": {
      "id": "ord_8899",
      "data": { "amount": 199.99 }
    }
  }'`,
                  },
                  {
                    language: 'JSON',
                    code: `// HTTP 200 OK Response
{
  "transformed_payload": {
    "order_id": "ord_8899",
    "total_usd": 199.99
  }
}`,
                  },
                ]}
              />
            </div>
          </div>
        )}

        {/* SECTION: DLQ API */}
        {activeSection === 'dlq-api' && (
          <div className="space-y-6 animate-in fade-in">
            <div>
              <span className="text-xs font-mono text-emerald-400 font-semibold uppercase">Quarantine</span>
              <h1 className="text-3xl font-extrabold text-white mt-1 tracking-tight">Dead Letter Queue (DLQ) API</h1>
              <p className="text-sm text-zinc-400 mt-2 leading-relaxed">
                Inspect exhausted deliveries, replay failed items individually, or trigger bulk re-enqueue operations.
              </p>
            </div>

            <div className="space-y-6">
              <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-3">
                <div className="flex items-center space-x-2.5">
                  <span className="px-2 py-0.5 rounded text-xs font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                    POST
                  </span>
                  <span className="font-mono text-sm text-white font-bold">/api/v1/dlq/retry-all</span>
                </div>
                <p className="text-xs text-zinc-300">
                  Bulk requeue all dead-lettered webhook deliveries across the tenant for immediate retry.
                </p>
                <CodeBlock
                  title="Bulk Requeue DLQ"
                  singleLang="bash"
                  singleCode={`curl -X POST http://localhost:3001/api/v1/dlq/retry-all \\
  -H "Authorization: Bearer <TOKEN>"`}
                />
              </div>
            </div>
          </div>
        )}

        {/* SECTION: Health & Telemetry */}
        {activeSection === 'health-metrics' && (
          <div className="space-y-6 animate-in fade-in">
            <div>
              <span className="text-xs font-mono text-emerald-400 font-semibold uppercase">Observability</span>
              <h1 className="text-3xl font-extrabold text-white mt-1 tracking-tight">Healthz & Prometheus Metrics</h1>
              <p className="text-sm text-zinc-400 mt-2 leading-relaxed">
                Production liveness probes and real-time Prometheus telemetry metrics for Kubernetes, Grafana, and Datadog.
              </p>
            </div>

            <div className="space-y-6">
              <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-3">
                <div className="flex items-center space-x-2.5">
                  <span className="px-2 py-0.5 rounded text-xs font-mono font-bold bg-blue-500/10 text-blue-400 border border-blue-500/20">
                    GET
                  </span>
                  <span className="font-mono text-sm text-white font-bold">/healthz</span>
                </div>
                <p className="text-xs text-zinc-300">
                  Validates PostgreSQL connectivity and Redis worker queue responsiveness.
                </p>
                <CodeBlock
                  title="Health Probe"
                  snippets={[
                    {
                      language: 'cURL',
                      code: `curl -s http://localhost:3001/healthz`,
                    },
                    {
                      language: 'JSON',
                      code: `{
  "db": "ok",
  "queue": "ok"
}`,
                    },
                  ]}
                />
              </div>

              <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-3">
                <div className="flex items-center space-x-2.5">
                  <span className="px-2 py-0.5 rounded text-xs font-mono font-bold bg-blue-500/10 text-blue-400 border border-blue-500/20">
                    GET
                  </span>
                  <span className="font-mono text-sm text-white font-bold">/metrics</span>
                </div>
                <p className="text-xs text-zinc-300">
                  Prometheus metrics output with request durations, delivery counters, and circuit breaker trip counts.
                </p>
                <CodeBlock
                  title="Prometheus Metrics"
                  singleLang="bash"
                  singleCode={`curl -s http://localhost:3001/metrics`}
                />
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
};
