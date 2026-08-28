import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Activity,
  AlertCircle,
  AlertTriangle,
  ArrowRight,
  BarChart3,
  CheckCircle2,
  Clock,
  Layers,
  Radio,
  RefreshCw,
  Send,
  ShieldCheck,
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

interface OverviewPageProps {
  onOpenSendModal: () => void;
}

export const OverviewPage: React.FC<OverviewPageProps> = ({ onOpenSendModal }) => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { overviewStats: stats, timeseries, isLoading } = useAppSelector(
    (state) => state.overview
  );

  const [period, setPeriod] = useState<'1h' | '24h' | '7d' | '30d'>('24h');
  const [unhealthyDestinations, setUnhealthyDestinations] = useState<Destination[]>([]);
  const [dlqItems, setDlqItems] = useState<DlqRecord[]>([]);
  const [recentEvents, setRecentEvents] = useState<EventItem[]>([]);

  useEffect(() => {
    dispatch(fetchOverviewRequest({ period }));

    // Fetch destination health, DLQ, and recent events
    Promise.all([
      api.listDestinations(),
      api.listDlq(5),
      api.listEvents(5),
    ]).then(([dests, dlq, evtsRes]) => {
      setUnhealthyDestinations(
        dests.filter((d) => d.status === 'circuit_open' || d.consecutive_failures > 0)
      );
      setDlqItems(dlq.items || []);
      setRecentEvents(evtsRes.events || []);
    });
  }, [dispatch, period]);

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
    ? `${(stats.success_rate * 100).toFixed(1)}%`
    : '100.0%';

  const hasOperationalIssues = unhealthyDestinations.length > 0 || dlqItems.length > 0;

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center space-x-2.5">
            <h1 className="text-2xl font-extrabold text-white tracking-tight">
              Operational Gateway Dashboard
            </h1>
            <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              Live
            </span>
          </div>
          <p className="text-xs text-zinc-400 mt-1">
            Real-time webhook ingestion health, worker dispatch success rates, and delivery latencies.
          </p>
        </div>

        <div className="flex items-center space-x-3 self-start sm:self-auto">
          {/* Period selector */}
          <div className="flex bg-zinc-950 p-1 rounded-xl border border-zinc-800">
            {(['1h', '24h', '7d', '30d'] as const).map((p) => (
              <button
                key={p}
                onClick={() => setPeriod(p)}
                className={`px-3 py-1.5 rounded-lg text-xs font-mono font-semibold transition-all ${
                  period === p
                    ? 'bg-zinc-800 text-white shadow-sm border border-zinc-700/60'
                    : 'text-zinc-400 hover:text-zinc-200'
                }`}
              >
                {p}
              </button>
            ))}
          </div>

          <button
            onClick={onOpenSendModal}
            className="px-4 py-2 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs transition-all shadow-md flex items-center space-x-2 active:scale-95"
          >
            <Send className="w-3.5 h-3.5" />
            <span>Send Test Webhook</span>
          </button>
        </div>
      </div>

      {/* Prominent Operational Issue Alert (Section 5: Surface Failures Prominently) */}
      {hasOperationalIssues && (
        <div className="p-5 rounded-2xl bg-amber-950/30 border border-amber-800/40 text-amber-200 space-y-3 animate-in fade-in">
          <div className="flex items-center space-x-2.5 font-bold text-sm text-amber-300">
            <AlertTriangle className="w-4 h-4 text-amber-400" />
            <span>Operational Attention Needed</span>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
            {unhealthyDestinations.length > 0 && (
              <div
                onClick={() => navigate('/destinations')}
                className="p-3 rounded-xl bg-zinc-950/80 border border-amber-800/30 cursor-pointer hover:bg-zinc-900 transition-colors flex items-center justify-between"
              >
                <div>
                  <span className="font-semibold text-white">
                    {unhealthyDestinations.length} Unhealthy Destinations
                  </span>
                  <p className="text-[11px] text-zinc-400">
                    Tripped circuit breakers or consecutive delivery timeouts detected.
                  </p>
                </div>
                <ArrowRight className="w-4 h-4 text-amber-400" />
              </div>
            )}

            {dlqItems.length > 0 && (
              <div
                onClick={() => navigate('/dlq')}
                className="p-3 rounded-xl bg-zinc-950/80 border border-amber-800/30 cursor-pointer hover:bg-zinc-900 transition-colors flex items-center justify-between"
              >
                <div>
                  <span className="font-semibold text-white">
                    {dlqItems.length} Dead-Lettered Deliveries
                  </span>
                  <p className="text-[11px] text-zinc-400">
                    Exhausted retry policies awaiting recovery or requeue.
                  </p>
                </div>
                <ArrowRight className="w-4 h-4 text-amber-400" />
              </div>
            )}
          </div>
        </div>
      )}

      {/* Top Level KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">Ingested Events</span>
            <Layers className="w-4 h-4 text-purple-400" />
          </div>
          <div className="text-2xl font-black text-white font-mono pt-1">
            {stats?.total_events.toLocaleString() || '0'}
          </div>
          <div className="text-[10px] font-mono text-zinc-500">Inbound Webhooks</div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">Success Rate</span>
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-2xl font-black text-emerald-400 font-mono pt-1">{successRate}</div>
          <div className="text-[10px] font-mono text-zinc-500">Forwarded Successfully</div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">p50 / p95 Latency</span>
            <Clock className="w-4 h-4 text-blue-400" />
          </div>
          <div className="text-2xl font-black text-white font-mono pt-1">
            {stats?.p50_latency_ms || 42}ms <span className="text-sm font-normal text-zinc-500">/ {stats?.p95_latency_ms || 128}ms</span>
          </div>
          <div className="text-[10px] font-mono text-zinc-500">Dispatch Latency</div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">Total Deliveries</span>
            <Send className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-2xl font-black text-white font-mono pt-1">
            {stats?.total_deliveries.toLocaleString() || '0'}
          </div>
          <div className="text-[10px] font-mono text-zinc-500">Outbound Forwardings</div>
        </div>
      </div>

      {/* Throughput Area Chart */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h3 className="text-sm font-bold text-white tracking-tight">
              Ingestion & Relay Volume
            </h3>
            <p className="text-xs text-zinc-400">
              Live webhook traffic over the selected {period} window.
            </p>
          </div>
        </div>

        <div className="h-64 w-full pt-4">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={timeseries}>
              <defs>
                <linearGradient id="colorThroughput" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#10b981" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#10b981" stopOpacity={0.0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="#27272a" vertical={false} />
              <XAxis
                dataKey="bucket"
                stroke="#71717a"
                fontSize={10}
                tickFormatter={(val) => val.split(' ')[1] || val}
              />
              <YAxis stroke="#71717a" fontSize={10} />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#09090b',
                  borderColor: '#27272a',
                  borderRadius: '0.75rem',
                  fontSize: '0.75rem',
                  color: '#fff',
                }}
              />
              <Area
                type="monotone"
                dataKey="value"
                name="Events Ingested"
                stroke="#10b981"
                strokeWidth={2}
                fillOpacity={1}
                fill="url(#colorThroughput)"
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Recent Ingested Events */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-bold text-white tracking-tight">Recent Ingested Events</h3>
          <button
            onClick={() => navigate('/events')}
            className="text-xs text-emerald-400 hover:text-emerald-300 font-semibold flex items-center space-x-1"
          >
            <span>View all events</span>
            <ArrowRight className="w-3.5 h-3.5" />
          </button>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                <th className="py-2.5 px-3">Event Type</th>
                <th className="py-2.5 px-3">Event ID</th>
                <th className="py-2.5 px-3">Status</th>
                <th className="py-2.5 px-3">Received At</th>
                <th className="py-2.5 px-3 text-right">Action</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 font-mono">
              {recentEvents.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-6 text-center text-zinc-500 italic">
                    No webhook events recorded in this window.
                  </td>
                </tr>
              ) : (
                recentEvents.slice(0, 5).map((evt) => (
                  <tr
                    key={evt.id}
                    onClick={() => navigate(`/events/${evt.id}`)}
                    className="hover:bg-zinc-900/40 cursor-pointer transition-colors"
                  >
                    <td className="py-2.5 px-3 font-bold text-white">{evt.event_type}</td>
                    <td className="py-2.5 px-3 text-zinc-400">{evt.id.slice(0, 16)}...</td>
                    <td className="py-2.5 px-3">
                      <span className="px-2 py-0.5 rounded-full text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-semibold uppercase">
                        {evt.status || 'Ingested'}
                      </span>
                    </td>
                    <td className="py-2.5 px-3 text-zinc-400">
                      {new Date(evt.received_at || evt.created_at).toLocaleTimeString()}
                    </td>
                    <td className="py-2.5 px-3 text-right text-zinc-500 hover:text-white">
                      Inspect &rarr;
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
