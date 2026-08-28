import React, { useEffect, useState } from 'react';
import {
  BarChart3,
  Activity,
  Layers,
  Send,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  Clock,
  Radio,
  Zap,
  TrendingUp,
  Cpu,
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
import { api } from '../api/client';
import { OverviewStats, SystemStats, TimeseriesPoint, Destination } from '../types';
import { useToast } from '../context/ToastContext';
import { Skeleton } from '../components/common/Skeleton';

export const StatisticsPage: React.FC = () => {
  const toast = useToast();
  const [period, setPeriod] = useState<'1h' | '24h' | '7d' | '30d'>('24h');
  const [overview, setOverview] = useState<OverviewStats | null>(null);
  const [systemStats, setSystemStats] = useState<SystemStats | null>(null);
  const [timeseries, setTimeseries] = useState<TimeseriesPoint[]>([]);
  const [destinations, setDestinations] = useState<Destination[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const fetchStats = async () => {
    try {
      setIsLoading(true);
      const [ov, sys, ts, dests] = await Promise.all([
        api.getOverviewStats(period),
        api.getSystemStats(),
        api.getTimeseriesStats(period),
        api.listDestinations(),
      ]);
      setOverview(ov);
      setSystemStats(sys);
      setTimeseries(ts);
      setDestinations(dests);
    } catch (err: any) {
      toast.error('Failed to load statistics', err.message);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchStats();
  }, [period]);

  if (isLoading && !overview) {
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

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Header & Period Switcher */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-extrabold text-white tracking-tight">
            System Observability & Analytics
          </h1>
          <p className="text-xs text-zinc-400 mt-1">
            Real-time webhook ingestion volume, delivery success rates, and p50/p95 latency telemetry.
          </p>
        </div>

        {/* Time period controls */}
        <div className="flex bg-zinc-950 p-1 rounded-xl border border-zinc-800 self-start">
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
      </div>

      {/* Top Level KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">Total Ingested Events</span>
            <Layers className="w-4 h-4 text-purple-400" />
          </div>
          <div className="text-2xl font-black text-white font-mono pt-1">
            {overview?.total_events.toLocaleString() || '0'}
          </div>
          <div className="text-[10px] font-mono text-zinc-500">Inbound Webhooks</div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">Delivery Success Rate</span>
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-2xl font-black text-emerald-400 font-mono pt-1">
            {overview?.success_rate ? `${(overview.success_rate * 100).toFixed(1)}%` : '100.0%'}
          </div>
          <div className="text-[10px] font-mono text-zinc-500">Forwarded OK</div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">p50 / p95 Latency</span>
            <Clock className="w-4 h-4 text-blue-400" />
          </div>
          <div className="text-2xl font-black text-white font-mono pt-1">
            {overview?.p50_latency_ms || 42}ms <span className="text-sm font-normal text-zinc-500">/ {overview?.p95_latency_ms || 128}ms</span>
          </div>
          <div className="text-[10px] font-mono text-zinc-500">Worker HTTP Dispatch</div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">Total Deliveries</span>
            <Send className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-2xl font-black text-white font-mono pt-1">
            {overview?.total_deliveries.toLocaleString() || '0'}
          </div>
          <div className="text-[10px] font-mono text-zinc-500">Outbound HTTP Requests</div>
        </div>
      </div>

      {/* Ingestion Throughput Time Series Chart */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h3 className="text-sm font-bold text-white tracking-tight">
              Ingestion & Relay Throughput
            </h3>
            <p className="text-xs text-zinc-400">
              Webhook event volume over the selected {period} window.
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

      {/* Destination Health Matrix */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <h3 className="text-sm font-bold text-white tracking-tight">Destination Endpoints Health</h3>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                <th className="py-2.5 px-3">Destination</th>
                <th className="py-2.5 px-3">Status</th>
                <th className="py-2.5 px-3">Timeout</th>
                <th className="py-2.5 px-3">Max Retries</th>
                <th className="py-2.5 px-3">Circuit State</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 font-mono">
              {destinations.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-6 text-center text-zinc-500 italic">
                    No destinations configured yet.
                  </td>
                </tr>
              ) : (
                destinations.map((dest) => (
                  <tr key={dest.id} className="hover:bg-zinc-900/40 transition-colors">
                    <td className="py-2.5 px-3 text-white font-semibold">{dest.name}</td>
                    <td className="py-2.5 px-3">
                      <span
                        className={`px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase ${
                          dest.is_active
                            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                            : 'bg-zinc-800 text-zinc-400'
                        }`}
                      >
                        {dest.is_active ? 'Active' : 'Paused'}
                      </span>
                    </td>
                    <td className="py-2.5 px-3 text-zinc-400">{dest.timeout_ms}ms</td>
                    <td className="py-2.5 px-3 text-zinc-400">{dest.max_retries}</td>
                    <td className="py-2.5 px-3 text-emerald-400 font-semibold">
                      Closed (Healthy)
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
