import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import {
  ArrowRight,
  BookOpen,
  Check,
  CheckCircle2,
  Code2,
  Copy,
  Cpu,
  Flame,
  Globe,
  Layers,
  Lock,
  Radio,
  RefreshCw,
  Server,
  ShieldCheck,
  Sparkles,
  Zap,
} from 'lucide-react';
import { PublicLayout } from '../../components/public/PublicLayout';
import { PLANS } from '../../config/plans';
import { useAppSelector } from '../../store/hooks';

export const LandingPage: React.FC = () => {
  const { user, token } = useAppSelector((state) => state.auth);
  const isAuthenticated = !!(user && token);

  const [activeCodeTab, setActiveCodeTab] = useState<'curl' | 'nodejs' | 'python' | 'go'>('curl');
  const [copied, setCopied] = useState(false);

  const codeSnippets = {
    curl: `curl -X POST http://localhost:3001/hooks/stripe-inbound \\
  -H "Content-Type: application/json" \\
  -H "X-Event-Type: payment.succeeded" \\
  -d '{
    "id": "evt_3MjjkwLkdIwHu7ix",
    "amount": 4999,
    "currency": "usd",
    "customer": "cus_99881122"
  }'`,
    nodejs: `import { RelayCoreClient } from '@relaycore/sdk';

const relay = new RelayCoreClient({
  baseUrl: 'http://localhost:3001',
  apiKey: process.env.RELAYCORE_API_KEY,
});

await relay.sendWebhook('stripe-inbound', {
  id: 'evt_3MjjkwLkdIwHu7ix',
  amount: 4999,
  currency: 'usd',
}, 'payment.succeeded');`,
    python: `import requests

res = requests.post(
    "http://localhost:3001/hooks/stripe-inbound",
    json={"id": "evt_3MjjkwLkdIwHu7ix", "amount": 4999, "currency": "usd"},
    headers={"X-Event-Type": "payment.succeeded"}
)
print("Queued Event ID:", res.json()["id"])`,
    go: `package main

import (
    "bytes"
    "net/http"
)

func main() {
    payload := []byte(\`{"id":"evt_3MjjkwLkdIwHu7ix","amount":4999,"currency":"usd"}\`)
    req, _ := http.NewRequest("POST", "http://localhost:3001/hooks/stripe-inbound", bytes.NewBuffer(payload))
    req.Header.Set("Content-Type", "application/json")
    req.Header.Set("X-Event-Type", "payment.succeeded")
    http.DefaultClient.Do(req)
}`,
  };

  const handleCopyCode = () => {
    navigator.clipboard.writeText(codeSnippets[activeCodeTab]);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <PublicLayout>
      <div className="space-y-24 sm:space-y-32 py-12 sm:py-20 animate-in fade-in duration-200 overflow-hidden">
        {/* HERO SECTION */}
        <section className="max-w-7xl mx-auto px-4 sm:px-8 text-center space-y-8 relative">
          {/* Ambient Glow */}
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[350px] bg-emerald-500/10 blur-[120px] rounded-full pointer-events-none -z-10" />

          <div className="inline-flex items-center space-x-2 px-3.5 py-1.5 rounded-full text-xs font-mono font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 shadow-inner">
            <Sparkles className="w-3.5 h-3.5" />
            <span>Developer-First Webhook & Event Relay PaaS</span>
          </div>

          <div className="max-w-4xl mx-auto space-y-4">
            <h1 className="text-4xl sm:text-6xl font-extrabold text-white tracking-tight leading-[1.1]">
              Reliable Webhook and Event Delivery Infrastructure
            </h1>
            <p className="text-base sm:text-lg text-zinc-400 max-w-2xl mx-auto leading-relaxed">
              Ingest, cryptographically verify, transform, and fan-out webhooks to your downstream destinations with automated retries, circuit breakers, and zero dropped events.
            </p>
          </div>

          {/* Action CTAs */}
          <div className="flex flex-wrap items-center justify-center gap-4 pt-2">
            <Link
              to={isAuthenticated ? '/dashboard' : '/signup'}
              className="inline-flex items-center space-x-2 px-6 py-3 rounded-2xl text-sm font-bold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-xl shadow-emerald-500/20 transition-all hover:scale-105 active:scale-95 group"
            >
              <span>{isAuthenticated ? 'Go to Dashboard' : 'Start Building Free'}</span>
              <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
            </Link>

            <Link
              to="/docs"
              className="inline-flex items-center space-x-2 px-6 py-3 rounded-2xl text-sm font-semibold bg-zinc-900/90 hover:bg-zinc-800 text-zinc-200 border border-zinc-800 hover:border-zinc-700 transition-all shadow-md"
            >
              <BookOpen className="w-4 h-4 text-zinc-400" />
              <span>Read Documentation</span>
            </Link>
          </div>

          <div className="pt-4 flex items-center justify-center space-x-6 text-xs text-zinc-500 font-mono">
            <span className="flex items-center space-x-1.5">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              <span>25,000 Free Events/mo</span>
            </span>
            <span className="flex items-center space-x-1.5">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              <span>No Credit Card Required</span>
            </span>
            <span className="flex items-center space-x-1.5 hidden sm:flex">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              <span>Sub-5ms Ingestion</span>
            </span>
          </div>

          {/* Interactive Code Playground / Terminal Preview */}
          <div className="max-w-3xl mx-auto pt-8 text-left">
            <div className="rounded-2xl bg-[#0c0c0e] border border-zinc-800 shadow-2xl overflow-hidden">
              {/* Terminal Tab Header */}
              <div className="flex items-center justify-between px-4 py-2.5 bg-zinc-950 border-b border-zinc-800/80">
                <div className="flex items-center space-x-2">
                  <span className="w-3 h-3 rounded-full bg-rose-500/60" />
                  <span className="w-3 h-3 rounded-full bg-amber-500/60" />
                  <span className="w-3 h-3 rounded-full bg-emerald-500/60" />
                  <span className="text-[11px] font-mono text-zinc-500 ml-2 font-medium">
                    POST /hooks/:slug (Public Ingestion)
                  </span>
                </div>

                <div className="flex items-center space-x-1">
                  {(['curl', 'nodejs', 'python', 'go'] as const).map((tab) => (
                    <button
                      key={tab}
                      type="button"
                      onClick={() => setActiveCodeTab(tab)}
                      className={`px-2.5 py-1 rounded-lg text-[11px] font-mono font-medium transition-colors ${
                        activeCodeTab === tab
                          ? 'bg-zinc-800 text-white font-semibold'
                          : 'text-zinc-500 hover:text-zinc-300'
                      }`}
                    >
                      {tab}
                    </button>
                  ))}
                  <button
                    type="button"
                    onClick={handleCopyCode}
                    aria-label="Copy snippet"
                    className="p-1.5 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800 ml-2 transition-colors"
                  >
                    {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                  </button>
                </div>
              </div>

              {/* Terminal Body */}
              <div className="p-5 font-mono text-xs text-zinc-200 overflow-x-auto leading-relaxed bg-[#0a0a0c]">
                <pre className="!bg-transparent !p-0 !m-0">
                  <code>{codeSnippets[activeCodeTab]}</code>
                </pre>
              </div>

              {/* Terminal Footer Status */}
              <div className="px-4 py-2 bg-zinc-950/90 border-t border-zinc-800/60 flex items-center justify-between text-[11px] font-mono text-zinc-500">
                <span className="flex items-center space-x-1.5 text-emerald-400">
                  <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
                  <span>202 Accepted • Ingested in 2.8ms</span>
                </span>
                <span>Payload encrypted at rest</span>
              </div>
            </div>
          </div>
        </section>

        {/* HOW IT WORKS SECTION */}
        <section className="max-w-7xl mx-auto px-4 sm:px-8 space-y-12">
          <div className="text-center space-y-3 max-w-2xl mx-auto">
            <span className="text-xs font-mono font-bold text-emerald-400 uppercase tracking-wider">
              Architecture Pipeline
            </span>
            <h2 className="text-3xl font-extrabold text-white tracking-tight">
              How RelayCore Dispatches Events
            </h2>
            <p className="text-xs sm:text-sm text-zinc-400 leading-relaxed">
              A high-throughput pipeline designed to isolate upstream third-party spikes from downstream receiver availability.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 relative">
            {/* Step 1 */}
            <div className="p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-3">
              <div className="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center font-bold font-mono">
                1
              </div>
              <h3 className="text-sm font-bold text-white">Ingest & Verify</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Receive HTTP POST at <code className="text-emerald-400 font-mono">/hooks/:slug</code>. Validate HMAC signatures in constant time and return 202 in &lt;5ms.
              </p>
            </div>

            {/* Step 2 */}
            <div className="p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-3">
              <div className="w-10 h-10 rounded-xl bg-blue-500/10 border border-blue-500/20 text-blue-400 flex items-center justify-center font-bold font-mono">
                2
              </div>
              <h3 className="text-sm font-bold text-white">Subscription Matching</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Evaluate wildcard filters (<code className="text-blue-400 font-mono">payment.*</code>) and fan out to all subscribed target destination endpoints.
              </p>
            </div>

            {/* Step 3 */}
            <div className="p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-3">
              <div className="w-10 h-10 rounded-xl bg-purple-500/10 border border-purple-500/20 text-purple-400 flex items-center justify-center font-bold font-mono">
                3
              </div>
              <h3 className="text-sm font-bold text-white">Transform & Deliver</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Apply JSONPath transformations and dispatch asynchronous HTTP requests via Tokio workers with timeout limits.
              </p>
            </div>

            {/* Step 4 */}
            <div className="p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-3">
              <div className="w-10 h-10 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-400 flex items-center justify-center font-bold font-mono">
                4
              </div>
              <h3 className="text-sm font-bold text-white">Retries & DLQ</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Exponential backoff with jitter on 5xx errors. Quarantines exhausted deliveries to the Dead Letter Queue for 1-click replay.
              </p>
            </div>
          </div>
        </section>

        {/* CORE FEATURES GRID */}
        <section className="max-w-7xl mx-auto px-4 sm:px-8 space-y-12">
          <div className="text-center space-y-3 max-w-2xl mx-auto">
            <span className="text-xs font-mono font-bold text-emerald-400 uppercase tracking-wider">
              Core Capabilities
            </span>
            <h2 className="text-3xl font-extrabold text-white tracking-tight">
              Enterprise Resilience Out of the Box
            </h2>
            <p className="text-xs sm:text-sm text-zinc-400 leading-relaxed">
              Engineered in Rust with strict multi-tenant isolation, cryptographic security, and sub-millisecond execution.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800/80 space-y-3 hover:border-zinc-700 transition-colors">
              <div className="w-10 h-10 rounded-xl bg-emerald-500/10 text-emerald-400 flex items-center justify-center">
                <ShieldCheck className="w-5 h-5" />
              </div>
              <h3 className="text-sm font-bold text-white">Constant-Time HMAC Signatures</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Native cryptographic verification for Stripe, GitHub, Shopify, and custom HMAC-SHA256 headers with timestamp tolerance replay attack defense.
              </p>
            </div>

            <div className="p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800/80 space-y-3 hover:border-zinc-700 transition-colors">
              <div className="w-10 h-10 rounded-xl bg-blue-500/10 text-blue-400 flex items-center justify-center">
                <Cpu className="w-5 h-5" />
              </div>
              <h3 className="text-sm font-bold text-white">Automated Circuit Breakers</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Protects downstream servers during outages. Automatically trips open on consecutive timeouts and recovers via canary probe deliveries.
              </p>
            </div>

            <div className="p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800/80 space-y-3 hover:border-zinc-700 transition-colors">
              <div className="w-10 h-10 rounded-xl bg-purple-500/10 text-purple-400 flex items-center justify-center">
                <Code2 className="w-5 h-5" />
              </div>
              <h3 className="text-sm font-bold text-white">JSONPath Transformation Engine</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Reshape and map inbound payloads dynamically before downstream delivery without writing or maintaining custom glue microservices.
              </p>
            </div>

            <div className="p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800/80 space-y-3 hover:border-zinc-700 transition-colors">
              <div className="w-10 h-10 rounded-xl bg-amber-500/10 text-amber-400 flex items-center justify-center">
                <RefreshCw className="w-5 h-5" />
              </div>
              <h3 className="text-sm font-bold text-white">Dead Letter Queue & Replay</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Quarantine failed deliveries with complete HTTP status codes, latency in milliseconds, and response snippets for 1-click batch replays.
              </p>
            </div>

            <div className="p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800/80 space-y-3 hover:border-zinc-700 transition-colors">
              <div className="w-10 h-10 rounded-xl bg-cyan-500/10 text-cyan-400 flex items-center justify-center">
                <Radio className="w-5 h-5" />
              </div>
              <h3 className="text-sm font-bold text-white">Keyset Cursor Pagination</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Inspect millions of events and delivery traces with fast O(1) indexed cursor pagination without database query degradation.
              </p>
            </div>

            <div className="p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800/80 space-y-3 hover:border-zinc-700 transition-colors">
              <div className="w-10 h-10 rounded-xl bg-rose-500/10 text-rose-400 flex items-center justify-center">
                <Lock className="w-5 h-5" />
              </div>
              <h3 className="text-sm font-bold text-white">Multi-Tenant Isolation</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                PostgreSQL row-level tenant partitioning, AES-256-GCM secret encryption at rest, and scoped API keys (read_only / full).
              </p>
            </div>
          </div>
        </section>

        {/* PRICING PREVIEW SECTION */}
        <section className="max-w-7xl mx-auto px-4 sm:px-8 space-y-12">
          <div className="text-center space-y-3 max-w-2xl mx-auto">
            <span className="text-xs font-mono font-bold text-emerald-400 uppercase tracking-wider">
              Transparent Pricing
            </span>
            <h2 className="text-3xl font-extrabold text-white tracking-tight">
              Start Free, Scale as You Grow
            </h2>
            <p className="text-xs sm:text-sm text-zinc-400 leading-relaxed">
              Every newly registered tenant receives our Free Tier with 25,000 monthly events and full developer features.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-5xl mx-auto">
            {PLANS.slice(0, 3).map((plan) => (
              <div
                key={plan.id}
                className={`p-6 rounded-3xl bg-[#0e0e11] border transition-all flex flex-col justify-between space-y-6 ${
                  plan.highlight
                    ? 'border-emerald-500/80 shadow-2xl shadow-emerald-500/10 relative'
                    : 'border-zinc-800/80'
                }`}
              >
                {plan.badge && (
                  <span className="absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-0.5 rounded-full text-[10px] font-mono font-bold bg-emerald-500 text-zinc-950 uppercase tracking-wider">
                    {plan.badge}
                  </span>
                )}

                <div className="space-y-4">
                  <div className="space-y-1">
                    <h3 className="text-base font-bold text-white">{plan.name}</h3>
                    <p className="text-xs text-zinc-400 leading-relaxed">{plan.tagline}</p>
                  </div>

                  <div className="flex items-baseline space-x-1">
                    <span className="text-3xl font-extrabold text-white">
                      ${plan.priceMonthly}
                    </span>
                    <span className="text-xs text-zinc-500">/month</span>
                  </div>

                  <ul className="space-y-2 pt-2 border-t border-zinc-800/80 text-xs text-zinc-300">
                    {plan.features.slice(0, 5).map((f, idx) => (
                      <li key={idx} className="flex items-start space-x-2">
                        <Check className="w-3.5 h-3.5 text-emerald-400 shrink-0 mt-0.5" />
                        <span>{f}</span>
                      </li>
                    ))}
                  </ul>
                </div>

                <Link
                  to={isAuthenticated ? '/dashboard/billing' : '/signup'}
                  className={`w-full py-2.5 px-4 rounded-xl text-xs font-semibold text-center transition-all ${
                    plan.highlight
                      ? 'bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-bold shadow-md'
                      : 'bg-zinc-800 hover:bg-zinc-700 text-white'
                  }`}
                >
                  {plan.ctaText}
                </Link>
              </div>
            ))}
          </div>

          <div className="text-center">
            <Link
              to="/pricing"
              className="text-xs font-semibold text-emerald-400 hover:text-emerald-300 inline-flex items-center space-x-1"
            >
              <span>View detailed plan comparison and enterprise pricing</span>
              <ArrowRight className="w-3.5 h-3.5" />
            </Link>
          </div>
        </section>

        {/* BOTTOM CTA BANNER */}
        <section className="max-w-5xl mx-auto px-4 sm:px-8">
          <div className="p-8 sm:p-12 rounded-3xl bg-gradient-to-tr from-[#121215] to-[#0e0e11] border border-zinc-800 shadow-2xl text-center space-y-6 relative overflow-hidden">
            <div className="absolute -right-16 -top-16 w-64 h-64 bg-emerald-500/10 rounded-full blur-3xl pointer-events-none" />
            
            <div className="space-y-3 max-w-xl mx-auto">
              <h2 className="text-2xl sm:text-3xl font-extrabold text-white tracking-tight">
                Start Building with RelayCore Today
              </h2>
              <p className="text-xs sm:text-sm text-zinc-400 leading-relaxed">
                Connect your upstream webhooks and start streaming resilient deliveries in less than 5 minutes.
              </p>
            </div>

            <div className="flex flex-wrap items-center justify-center gap-3">
              <Link
                to={isAuthenticated ? '/dashboard' : '/signup'}
                className="inline-flex items-center space-x-2 px-6 py-3 rounded-xl text-xs font-bold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-lg shadow-emerald-500/20 transition-all font-sans"
              >
                <span>{isAuthenticated ? 'Go to Dashboard' : 'Create Free Account'}</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </Link>
              <Link
                to="/docs/getting-started/quickstart"
                className="inline-flex items-center space-x-2 px-6 py-3 rounded-xl text-xs font-semibold bg-zinc-900 hover:bg-zinc-800 text-zinc-200 border border-zinc-800 transition-all"
              >
                <span>Read the Quickstart</span>
              </Link>
            </div>
          </div>
        </section>
      </div>
    </PublicLayout>
  );
};
