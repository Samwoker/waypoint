import React, { useEffect, useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import {
  Activity,
  AlertCircle,
  AlertTriangle,
  ArrowRight,
  BarChart3,
  CheckCircle2,
  Clock,
  Key,
  Layers,
  Radio,
  RefreshCw,
  Send,
  ShieldCheck,
  Sparkles,
  Zap,
} from 'lucide-react';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchOverviewRequest } from '../store/slices/overviewSlice';
import { api } from '../api/client';
import { Destination, DlqRecord, EventItem } from '../types';
import { Skeleton } from '../components/common/Skeleton';
import { getPlan, formatEventLimit, getUsagePercentage } from '../config/plans';

interface OverviewPageProps {
  onOpenSendModal: () => void;
}

export const OverviewPage: React.FC<OverviewPageProps> = ({ onOpenSendModal }) => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { user, currentTenant } = useAppSelector((state) => state.auth);
  const { overviewStats: stats, timeseries, isLoading } = useAppSelector(
    (state) => state.overview
  );

  const [period, setPeriod] = useState<'1h' | '24h' | '7d' | '30d'>('24h');
  const [unhealthyDestinations, setUnhealthyDestinations] = useState<Destination[]>([]);
  const [dlqItems, setDlqItems] = useState<DlqRecord[]>([]);
  const [recentEvents, setRecentEvents] = useState<EventItem[]>([]);
  const [hasSources, setHasSources] = useState(false);
  const [hasDestinations, setHasDestinations] = useState(false);
  const [hasSubscriptions, setHasSubscriptions] = useState(false);

  useEffect(() => {
    dispatch(fetchOverviewRequest({ period }));

    // Fetch destination health, DLQ, recent events, and check onboarding state
    Promise.all([
      api.listDestinations(),
      api.listDlq(5),
      api.listEvents(5),
      api.listSources(),
      api.listSubscriptions(),
    ]).then(([dests, dlq, evtsRes, sourcesRes, subsRes]) => {
      setUnhealthyDestinations(
        dests.filter((d) => d.status === 'circuit_open' || d.consecutive_failures > 0)
      );
      setDlqItems(dlq.items || []);
      setRecentEvents(evtsRes.events || []);
      setHasDestinations(dests.length > 0);
      setHasSources(sourcesRes.length > 0);
      setHasSubscriptions(subsRes.length > 0);
    });
  }, [dispatch, period]);

  const currentPlan = getPlan('free');
  const totalEvents = stats?.total_events || 0;
  const usagePercent = getUsagePercentage(totalEvents, currentPlan.eventLimit);

  if (isLoading && !stats) {
    return (
      <div className="p-8 max-w-7xl mx-auto space-y-6">
        <Skeleton className="h-8 w-48" />
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <Skeleton key={i} className="h-28 rounded-2xl" />
          ))}
        </div>
        <Skeleton className="h-72 w-full rounded-2xl" />
      </div>
    );
  }

  const successRate = stats?.success_rate
    ? (Number(stats.success_rate) * 100).toFixed(1)
    : '100.0';

  const chartData = (timeseries || []).map((pt) => ({
    time: pt.bucket.split(' ')[1] || pt.bucket,
    events: Number(pt.value) || 0,
  }));

  const allOnboardingComplete =
    hasSources && hasDestinations && hasSubscriptions && recentEvents.length > 0;

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150 font-sans">
      {/* Top Header with Welcome & Plan Meter */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-zinc-800 pb-6">
        <div className="space-y-1">
          <div className="flex items-center space-x-2">
            <span className="text-xs font-mono text-zinc-500 uppercase tracking-wider">
              {currentTenant?.name || user?.email || 'Workspace Dashboard'}
            </span>
            <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              FREE PLAN
            </span>
          </div>
          <h1 className="text-2xl sm:text-3xl font-extrabold text-white tracking-tight">
            Overview & Telemetry
          </h1>
          <p className="text-xs text-zinc-400">
            Real-time event stream monitoring, delivery performance, and circuit breaker status.
          </p>
        </div>

        {/* Right: Quick Plan Meter & Actions */}
        <div className="flex items-center space-x-3">
          <Link
            to="/dashboard/usage"
            className="p-3 rounded-2xl bg-[#0e0e11] hover:bg-zinc-900 border border-zinc-800 transition-colors flex items-center space-x-3 text-xs"
          >
            <div>
              <div className="flex justify-between items-center text-[10px] font-mono text-zinc-400 space-x-2">
                <span>Monthly Events</span>
                <span className="text-white font-bold">{usagePercent}%</span>
              </div>
              <div className="w-24 h-1.5 bg-zinc-950 rounded-full mt-1.5 overflow-hidden border border-zinc-800">
                <div
                  className="h-full bg-emerald-500 rounded-full transition-all"
                  style={{ width: `${usagePercent}%` }}
                />
              </div>
            </div>
            <span className="text-zinc-500 hover:text-white font-mono text-xs">→</span>
          </Link>

          <button
            type="button"
            onClick={onOpenSendModal}
            className="flex items-center space-x-1.5 px-3.5 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-bold text-xs shadow-md shadow-emerald-500/10 transition-all active:scale-95"
          >
            <Send className="w-3.5 h-3.5" />
            <span>Send Test Webhook</span>
          </button>
        </div>
      </div>

      {/* NEW USER ONBOARDING WIZARD (Shows if not all resources created) */}
      {!allOnboardingComplete && (
        <div className="p-6 rounded-3xl bg-gradient-to-tr from-[#121215] to-[#0e0e11] border border-zinc-800/80 space-y-4 shadow-sm">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2.5">
              <Sparkles className="w-4 h-4 text-emerald-400" />
              <h3 className="text-sm font-bold text-white">Getting Started with RelayCore</h3>
            </div>
            <span className="text-xs font-mono text-zinc-500">
              Follow these 4 quick steps to stream live webhooks
            </span>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-4 gap-3">
            {/* Step 1: Create Source */}
            <Link
              to="/dashboard/sources"
              className={`p-4 rounded-2xl border transition-all flex flex-col justify-between space-y-2 ${
                hasSources
                  ? 'bg-emerald-950/10 border-emerald-800/40 text-zinc-300'
                  : 'bg-zinc-950 hover:bg-zinc-900 border-zinc-800 text-zinc-300'
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-white">1. Create Source</span>
                {hasSources ? (
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                ) : (
                  <Layers className="w-4 h-4 text-zinc-500" />
                )}
              </div>
              <p className="text-[11px] text-zinc-400">
                Generate your public <code className="text-emerald-400">/hooks/:slug</code> URL.
              </p>
            </Link>

            {/* Step 2: Create Destination */}
            <Link
              to="/dashboard/destinations"
              className={`p-4 rounded-2xl border transition-all flex flex-col justify-between space-y-2 ${
                hasDestinations
                  ? 'bg-emerald-950/10 border-emerald-800/40 text-zinc-300'
                  : 'bg-zinc-950 hover:bg-zinc-900 border-zinc-800 text-zinc-300'
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-white">2. Register Destination</span>
                {hasDestinations ? (
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                ) : (
                  <Radio className="w-4 h-4 text-zinc-500" />
                )}
              </div>
              <p className="text-[11px] text-zinc-400">
                Enter your receiver URL & timeout threshold.
              </p>
            </Link>

            {/* Step 3: Create Subscription */}
            <Link
              to="/dashboard/subscriptions"
              className={`p-4 rounded-2xl border transition-all flex flex-col justify-between space-y-2 ${
                hasSubscriptions
                  ? 'bg-emerald-950/10 border-emerald-800/40 text-zinc-300'
                  : 'bg-zinc-950 hover:bg-zinc-900 border-zinc-800 text-zinc-300'
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-white">3. Create Subscription</span>
                {hasSubscriptions ? (
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                ) : (
                  <Activity className="w-4 h-4 text-zinc-500" />
                )}
              </div>
              <p className="text-[11px] text-zinc-400">
                Route events with wildcard filters (<code className="text-blue-400">payment.*</code>).
              </p>
            </Link>

            {/* Step 4: Send Test Webhook */}
            <div
              onClick={onOpenSendModal}
              className={`p-4 rounded-2xl border cursor-pointer transition-all flex flex-col justify-between space-y-2 ${
                recentEvents.length > 0
                  ? 'bg-emerald-950/10 border-emerald-800/40 text-zinc-300'
                  : 'bg-zinc-950 hover:bg-zinc-900 border-zinc-800 text-zinc-300'
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-white">4. Send Webhook</span>
                {recentEvents.length > 0 ? (
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                ) : (
                  <Send className="w-4 h-4 text-zinc-500" />
                )}
              </div>
              <p className="text-[11px] text-zinc-400">
                Dispatch an event & trace the delivery in real-time.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* KPI Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Total Events */}
        <div className="p-5 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono text-zinc-400 uppercase">Ingested Events</span>
            <div className="w-7 h-7 rounded-lg bg-emerald-500/10 text-emerald-400 flex items-center justify-center">
              <Zap className="w-4 h-4" />
            </div>
          </div>
          <div className="text-2xl font-extrabold text-white">
            {stats?.total_events?.toLocaleString() || 0}
          </div>
          <span className="text-[11px] font-mono text-zinc-500 block">
            Across active Inbound Sources
          </span>
        </div>

        {/* Deliveries */}
        <div className="p-5 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono text-zinc-400 uppercase">Total Deliveries</span>
            <div className="w-7 h-7 rounded-lg bg-blue-500/10 text-blue-400 flex items-center justify-center">
              <RefreshCw className="w-4 h-4" />
            </div>
          </div>
          <div className="text-2xl font-extrabold text-white">
            {stats?.total_deliveries?.toLocaleString() || 0}
          </div>
          <span className="text-[11px] font-mono text-zinc-500 block">
            {stats?.delivered_count?.toLocaleString() || 0} successfully delivered
          </span>
        </div>

        {/* Success Rate */}
        <div className="p-5 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono text-zinc-400 uppercase">Success Rate</span>
            <div className="w-7 h-7 rounded-lg bg-purple-500/10 text-purple-400 flex items-center justify-center">
              <CheckCircle2 className="w-4 h-4" />
            </div>
          </div>
          <div className="text-2xl font-extrabold text-emerald-400 font-mono">
            {successRate}%
          </div>
          <span className="text-[11px] font-mono text-zinc-500 block">
            P95 Latency: <strong className="text-zinc-300 font-bold">{stats?.p95_latency_ms || 45}ms</strong>
          </span>
        </div>

        {/* DLQ Count */}
        <div className="p-5 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono text-zinc-400 uppercase">Quarantined DLQ</span>
            <div className="w-7 h-7 rounded-lg bg-amber-500/10 text-amber-400 flex items-center justify-center">
              <AlertTriangle className="w-4 h-4" />
            </div>
          </div>
          <div className="text-2xl font-extrabold text-white font-mono">
            {dlqItems.length}
          </div>
          <span className="text-[11px] font-mono text-zinc-500 block">
            Exhausted retry budget
          </span>
        </div>
      </div>

      {/* Throughput Area Chart */}
      <div className="p-6 rounded-3xl bg-[#0e0e11] border border-zinc-800 space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <BarChart3 className="w-4 h-4 text-emerald-400" />
            <h3 className="text-sm font-bold text-white">Event Ingestion Throughput</h3>
          </div>
          <div className="flex bg-zinc-950 p-1 rounded-xl border border-zinc-800 text-[10px] font-mono">
            {(['1h', '24h', '7d', '30d'] as const).map((p) => (
              <button
                key={p}
                type="button"
                onClick={() => setPeriod(p)}
                className={`px-2.5 py-0.5 rounded-lg transition-colors ${
                  period === p ? 'bg-zinc-800 text-white font-bold' : 'text-zinc-500'
                }`}
              >
                {p}
              </button>
            ))}
          </div>
        </div>

        <div className="h-64 w-full">
          {chartData.length > 0 ? (
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
                <defs>
                  <linearGradient id="eventGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#10b981" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#10b981" stopOpacity={0.0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#27272a" opacity={0.5} />
                <XAxis dataKey="time" stroke="#71717a" fontSize={10} tickLine={false} />
                <YAxis stroke="#71717a" fontSize={10} tickLine={false} />
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#09090b',
                    borderColor: '#27272a',
                    borderRadius: '12px',
                    fontSize: '11px',
                    color: '#fff',
                  }}
                />
                <Area
                  type="monotone"
                  dataKey="events"
                  stroke="#10b981"
                  strokeWidth={2}
                  fillOpacity={1}
                  fill="url(#eventGradient)"
                />
              </AreaChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-full flex items-center justify-center text-xs text-zinc-500 font-mono">
              No throughput traffic recorded in this period.
            </div>
          )}
        </div>
      </div>

      {/* Recent Events & Deliveries Split */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Recent Events */}
        <div className="p-6 rounded-3xl bg-[#0e0e11] border border-zinc-800 space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-white">Recent Ingested Events</h3>
            <Link to="/dashboard/events" className="text-xs text-emerald-400 hover:underline">
              View all →
            </Link>
          </div>

          <div className="divide-y divide-zinc-800/60">
            {recentEvents.map((evt) => (
              <div
                key={evt.id}
                onClick={() => navigate(`/dashboard/events/${evt.id}`)}
                className="py-3 flex items-center justify-between hover:bg-zinc-900/40 cursor-pointer rounded-lg px-2 transition-colors text-xs"
              >
                <div className="space-y-0.5">
                  <div className="font-mono font-bold text-white">{evt.event_type}</div>
                  <div className="text-[11px] font-mono text-zinc-500">{evt.id.slice(0, 16)}...</div>
                </div>
                <span className="px-2 py-0.5 rounded text-[10px] font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                  {evt.status}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Quarantined DLQ or Destinations */}
        <div className="p-6 rounded-3xl bg-[#0e0e11] border border-zinc-800 space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-white">Dead Letter Queue (Quarantine)</h3>
            <Link to="/dashboard/dlq" className="text-xs text-emerald-400 hover:underline">
              View DLQ →
            </Link>
          </div>

          {dlqItems.length === 0 ? (
            <div className="py-8 text-center text-xs text-zinc-500 space-y-1">
              <CheckCircle2 className="w-8 h-8 text-emerald-500/40 mx-auto" />
              <p>No dead-lettered deliveries. All systems healthy.</p>
            </div>
          ) : (
            <div className="divide-y divide-zinc-800/60">
              {dlqItems.map((item) => (
                <div
                  key={item.delivery_id}
                  onClick={() => navigate('/dashboard/dlq')}
                  className="py-3 flex items-center justify-between hover:bg-zinc-900/40 cursor-pointer rounded-lg px-2 transition-colors text-xs"
                >
                  <div className="space-y-0.5">
                    <div className="font-mono font-bold text-rose-400">{item.event_type}</div>
                    <div className="text-[11px] text-zinc-500">{item.last_error || 'Retry budget exhausted'}</div>
                  </div>
                  <span className="px-2 py-0.5 rounded text-[10px] font-mono bg-rose-500/10 text-rose-400 border border-rose-500/20">
                    Failed
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
