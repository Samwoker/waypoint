import React from 'react';
import { Link } from 'react-router-dom';
import {
  ArrowRight,
  BookOpen,
  CheckCircle2,
  Code2,
  Cpu,
  Layers,
  Lock,
  Radio,
  RefreshCw,
  Server,
  ShieldCheck,
  Zap,
} from 'lucide-react';
import { PublicLayout } from '../../components/public/PublicLayout';
import { useAppSelector } from '../../store/hooks';

export const FeaturesPage: React.FC = () => {
  const { user, token } = useAppSelector((state) => state.auth);
  const isAuthenticated = !!(user && token);

  const features = [
    {
      icon: <Zap className="w-6 h-6 text-emerald-400" />,
      title: 'High-Throughput Non-Blocking Ingestion (<5ms)',
      description:
        'RelayCore validates incoming request signatures, writes an immutable audit record to PostgreSQL, enqueues delivery jobs to Redis, and returns 202 Accepted in under 5 milliseconds.',
      docsLink: '/docs/introduction/architecture',
    },
    {
      icon: <ShieldCheck className="w-6 h-6 text-blue-400" />,
      title: 'Cryptographic Constant-Time HMAC Verification',
      description:
        'Prevents timing attacks with constant-time equality comparisons. Built-in engines for Stripe v1 timestamped signatures, GitHub SHA-256 HMAC, Shopify Base64 HMAC, and custom secrets.',
      docsLink: '/docs/concepts/core-concepts',
    },
    {
      icon: <Radio className="w-6 h-6 text-purple-400" />,
      title: 'Wildcard Subscription Routing & Fan-Out',
      description:
        'Connect a single webhook source to multiple downstream internal services (Billing, Analytics, CRM). Filter events using wildcard pattern matching such as payment.* or invoice.paid.',
      docsLink: '/docs/api/subscriptions',
    },
    {
      icon: <RefreshCw className="w-6 h-6 text-amber-400" />,
      title: 'Exponential Backoff Retries & Jitter',
      description:
        'Guarantees at-least-once delivery. Automatically recalculates retry backoff intervals using randomized full jitter to prevent overwhelming recovering downstream servers.',
      docsLink: '/docs/concepts/retry-and-backoff',
    },
    {
      icon: <Cpu className="w-6 h-6 text-cyan-400" />,
      title: 'Automated Circuit Breaker Protection',
      description:
        'Monitors consecutive downstream timeouts and 5xx errors. Automatically trips open to pause new dispatches and uses canary probe deliveries to verify recovery.',
      docsLink: '/docs/concepts/circuit-breaker',
    },
    {
      icon: <Layers className="w-6 h-6 text-rose-400" />,
      title: 'Dead Letter Queue (DLQ) & 1-Click Replay',
      description:
        'Quarantines exhausted deliveries with complete HTTP status codes, latency in ms, error reasons, and response snippets. Replay individual items or trigger bulk retry operations.',
      docsLink: '/docs/api/dlq',
    },
    {
      icon: <Code2 className="w-6 h-6 text-emerald-400" />,
      title: 'JSONPath Transformation Sandbox',
      description:
        'Reshape and map inbound webhook payloads dynamically before downstream delivery. Test transformation templates in a real-time sandbox before attaching them to subscriptions.',
      docsLink: '/docs/api/transformations',
    },
    {
      icon: <Server className="w-6 h-6 text-blue-400" />,
      title: 'Prometheus Telemetry & Health Probes',
      description:
        'Production liveness probes at /healthz and detailed Prometheus metrics at /metrics for Grafana, Datadog, and Kubernetes autoscalers.',
      docsLink: '/docs/operations/monitoring',
    },
  ];

  return (
    <PublicLayout>
      <div className="space-y-20 py-12 sm:py-16 max-w-7xl mx-auto px-4 sm:px-8 animate-in fade-in duration-200">
        {/* Header */}
        <div className="text-center space-y-4 max-w-3xl mx-auto">
          <div className="inline-flex items-center space-x-2 px-3 py-1 rounded-full text-xs font-mono font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            <Zap className="w-3.5 h-3.5" />
            <span>Platform Capabilities</span>
          </div>

          <h1 className="text-3xl sm:text-5xl font-extrabold text-white tracking-tight">
            Built for Mission-Critical Webhook Delivery
          </h1>

          <p className="text-sm sm:text-base text-zinc-400 leading-relaxed">
            Every feature is engineered for high throughput, strict security, zero data loss, and complete operational visibility.
          </p>
        </div>

        {/* Features List */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          {features.map((feat, idx) => (
            <div
              key={idx}
              className="p-8 rounded-3xl bg-[#0e0e11] border border-zinc-800/80 space-y-4 hover:border-zinc-700 transition-all flex flex-col justify-between"
            >
              <div className="space-y-3">
                <div className="w-12 h-12 rounded-2xl bg-zinc-900 border border-zinc-800 flex items-center justify-center shadow-inner">
                  {feat.icon}
                </div>
                <h3 className="text-base font-bold text-white tracking-tight">
                  {feat.title}
                </h3>
                <p className="text-xs text-zinc-400 leading-relaxed">
                  {feat.description}
                </p>
              </div>

              <Link
                to={feat.docsLink}
                className="inline-flex items-center space-x-1.5 text-xs font-semibold text-emerald-400 hover:text-emerald-300 pt-2 group"
              >
                <span>Read technical documentation</span>
                <ArrowRight className="w-3.5 h-3.5 group-hover:translate-x-1 transition-transform" />
              </Link>
            </div>
          ))}
        </div>

        {/* CTA Banner */}
        <div className="p-8 sm:p-12 rounded-3xl bg-gradient-to-tr from-[#121215] to-[#0e0e11] border border-zinc-800 shadow-2xl text-center space-y-6">
          <h2 className="text-2xl sm:text-3xl font-extrabold text-white tracking-tight">
            Ready to Stream Webhooks Reliably?
          </h2>
          <p className="text-xs sm:text-sm text-zinc-400 max-w-xl mx-auto">
            Get started on the Free Tier with 25,000 monthly events and no credit card required.
          </p>
          <div className="flex justify-center space-x-3">
            <Link
              to={isAuthenticated ? '/dashboard' : '/signup'}
              className="inline-flex items-center space-x-2 px-6 py-3 rounded-xl text-xs font-bold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-md font-sans"
            >
              <span>{isAuthenticated ? 'Go to Dashboard' : 'Create Free Account'}</span>
              <ArrowRight className="w-3.5 h-3.5" />
            </Link>
          </div>
        </div>
      </div>
    </PublicLayout>
  );
};
