import React from 'react';
import { Link } from 'react-router-dom';
import {
  ArrowRight,
  BookOpen,
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
import { DocsLayout } from '../components/DocsLayout';

export const DocsLandingPage: React.FC = () => {
  return (
    <DocsLayout>
      <div className="space-y-12 animate-in fade-in duration-200">
        {/* Hero Section */}
        <div className="space-y-4 border-b border-zinc-800/80 pb-10">
          <div className="inline-flex items-center space-x-2 px-3 py-1 rounded-full text-xs font-mono font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            <Zap className="w-3.5 h-3.5" />
            <span>Developer Documentation</span>
          </div>

          <h1 className="text-3xl sm:text-4xl font-extrabold text-white tracking-tight">
            RelayCore Documentation
          </h1>

          <p className="text-base text-zinc-400 max-w-2xl leading-relaxed">
            Webhook ingestion, cryptographic verification, payload transformation, and resilient fan-out delivery infrastructure for your applications.
          </p>

          <div className="flex flex-wrap gap-3 pt-2">
            <Link
              to="/docs/getting-started/quickstart"
              className="inline-flex items-center space-x-2 px-4 py-2.5 rounded-xl text-xs font-bold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-lg shadow-emerald-500/10 transition-all group"
            >
              <span>Get Started</span>
              <ArrowRight className="w-3.5 h-3.5 group-hover:translate-x-0.5 transition-transform" />
            </Link>

            <Link
              to="/docs/api/overview"
              className="inline-flex items-center space-x-2 px-4 py-2.5 rounded-xl text-xs font-semibold bg-zinc-900 hover:bg-zinc-800 text-zinc-200 border border-zinc-800 transition-all"
            >
              <span>API Reference</span>
            </Link>

            <Link
              to="/docs/integrations/expressjs"
              className="inline-flex items-center space-x-2 px-4 py-2.5 rounded-xl text-xs font-semibold bg-zinc-900 hover:bg-zinc-800 text-zinc-200 border border-zinc-800 transition-all"
            >
              <span>Express.js Receiver</span>
            </Link>
          </div>
        </div>

        {/* Feature Grid / Core Entry Points */}
        <div className="space-y-6">
          <h2 className="text-lg font-bold text-white tracking-tight flex items-center space-x-2">
            <BookOpen className="w-4 h-4 text-emerald-400" />
            <span>Explore Documentation</span>
          </h2>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Card 1: Getting Started */}
            <Link
              to="/docs/getting-started/quickstart"
              className="group p-6 rounded-2xl bg-[#0e0e11] hover:bg-zinc-900/90 border border-zinc-800/80 hover:border-zinc-700 transition-all space-y-3 shadow-sm flex flex-col justify-between"
            >
              <div className="space-y-2">
                <div className="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center">
                  <Zap className="w-5 h-5" />
                </div>
                <h3 className="text-sm font-bold text-white group-hover:text-emerald-400 transition-colors">
                  5-Minute Quickstart
                </h3>
                <p className="text-xs text-zinc-400 leading-relaxed">
                  Start here to register a tenant, configure inbound sources, connect downstream receivers, and ingest your first live webhook.
                </p>
              </div>
              <div className="flex items-center space-x-1 text-xs font-semibold text-emerald-400 pt-2">
                <span>Start Building</span>
                <ArrowRight className="w-3.5 h-3.5 group-hover:translate-x-1 transition-transform" />
              </div>
            </Link>

            {/* Card 2: Core Concepts */}
            <Link
              to="/docs/concepts/core-concepts"
              className="group p-6 rounded-2xl bg-[#0e0e11] hover:bg-zinc-900/90 border border-zinc-800/80 hover:border-zinc-700 transition-all space-y-3 shadow-sm flex flex-col justify-between"
            >
              <div className="space-y-2">
                <div className="w-10 h-10 rounded-xl bg-blue-500/10 border border-blue-500/20 text-blue-400 flex items-center justify-center">
                  <Layers className="w-5 h-5" />
                </div>
                <h3 className="text-sm font-bold text-white group-hover:text-blue-400 transition-colors">
                  Core Concepts & Mental Model
                </h3>
                <p className="text-xs text-zinc-400 leading-relaxed">
                  Understand the relationship between Tenants, Sources, Events, Subscriptions, Deliveries, Attempts, and Dead Letter Queues.
                </p>
              </div>
              <div className="flex items-center space-x-1 text-xs font-semibold text-blue-400 pt-2">
                <span>Explore Concepts</span>
                <ArrowRight className="w-3.5 h-3.5 group-hover:translate-x-1 transition-transform" />
              </div>
            </Link>

            {/* Card 3: API Reference */}
            <Link
              to="/docs/api/overview"
              className="group p-6 rounded-2xl bg-[#0e0e11] hover:bg-zinc-900/90 border border-zinc-800/80 hover:border-zinc-700 transition-all space-y-3 shadow-sm flex flex-col justify-between"
            >
              <div className="space-y-2">
                <div className="w-10 h-10 rounded-xl bg-purple-500/10 border border-purple-500/20 text-purple-400 flex items-center justify-center">
                  <Code2 className="w-5 h-5" />
                </div>
                <h3 className="text-sm font-bold text-white group-hover:text-purple-400 transition-colors">
                  REST API Reference
                </h3>
                <p className="text-xs text-zinc-400 leading-relaxed">
                  Exhaustive reference for every RelayCore endpoint: Auth, Sources, Destinations, Subscriptions, Keyset Cursor Pagination, and DLQ.
                </p>
              </div>
              <div className="flex items-center space-x-1 text-xs font-semibold text-purple-400 pt-2">
                <span>Browse Endpoints</span>
                <ArrowRight className="w-3.5 h-3.5 group-hover:translate-x-1 transition-transform" />
              </div>
            </Link>

            {/* Card 4: Node & Express Integrations */}
            <Link
              to="/docs/integrations/expressjs"
              className="group p-6 rounded-2xl bg-[#0e0e11] hover:bg-zinc-900/90 border border-zinc-800/80 hover:border-zinc-700 transition-all space-y-3 shadow-sm flex flex-col justify-between"
            >
              <div className="space-y-2">
                <div className="w-10 h-10 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-400 flex items-center justify-center">
                  <Cpu className="w-5 h-5" />
                </div>
                <h3 className="text-sm font-bold text-white group-hover:text-amber-400 transition-colors">
                  SDKs & Framework Integrations
                </h3>
                <p className="text-xs text-zinc-400 leading-relaxed">
                  Production-grade Node.js and Express.js webhook receiver code with raw buffer HMAC validation and fast 200 acknowledgments.
                </p>
              </div>
              <div className="flex items-center space-x-1 text-xs font-semibold text-amber-400 pt-2">
                <span>View Integrations</span>
                <ArrowRight className="w-3.5 h-3.5 group-hover:translate-x-1 transition-transform" />
              </div>
            </Link>

            {/* Card 5: Architecture & Lifecycle */}
            <Link
              to="/docs/introduction/architecture"
              className="group p-6 rounded-2xl bg-[#0e0e11] hover:bg-zinc-900/90 border border-zinc-800/80 hover:border-zinc-700 transition-all space-y-3 shadow-sm flex flex-col justify-between"
            >
              <div className="space-y-2">
                <div className="w-10 h-10 rounded-xl bg-cyan-500/10 border border-cyan-500/20 text-cyan-400 flex items-center justify-center">
                  <Radio className="w-5 h-5" />
                </div>
                <h3 className="text-sm font-bold text-white group-hover:text-cyan-400 transition-colors">
                  System Architecture & Pipelines
                </h3>
                <p className="text-xs text-zinc-400 leading-relaxed">
                  Deep-dive into Rust Tokio async task pools, Redis work streams, non-blocking ingestion (&lt;5ms), and database ERDs.
                </p>
              </div>
              <div className="flex items-center space-x-1 text-xs font-semibold text-cyan-400 pt-2">
                <span>Inspect Architecture</span>
                <ArrowRight className="w-3.5 h-3.5 group-hover:translate-x-1 transition-transform" />
              </div>
            </Link>

            {/* Card 6: Operations & Deployment */}
            <Link
              to="/docs/operations/deployment"
              className="group p-6 rounded-2xl bg-[#0e0e11] hover:bg-zinc-900/90 border border-zinc-800/80 hover:border-zinc-700 transition-all space-y-3 shadow-sm flex flex-col justify-between"
            >
              <div className="space-y-2">
                <div className="w-10 h-10 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 flex items-center justify-center">
                  <Server className="w-5 h-5" />
                </div>
                <h3 className="text-sm font-bold text-white group-hover:text-rose-400 transition-colors">
                  Operations & Production Deployment
                </h3>
                <p className="text-xs text-zinc-400 leading-relaxed">
                  Docker Compose setup, Kubernetes scaling topologies, Prometheus telemetry metrics, and health probes.
                </p>
              </div>
              <div className="flex items-center space-x-1 text-xs font-semibold text-rose-400 pt-2">
                <span>Deploy to Production</span>
                <ArrowRight className="w-3.5 h-3.5 group-hover:translate-x-1 transition-transform" />
              </div>
            </Link>
          </div>
        </div>
      </div>
    </DocsLayout>
  );
};
