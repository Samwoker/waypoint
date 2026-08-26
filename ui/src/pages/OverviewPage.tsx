import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  AlertTriangle,
  ArrowUpRight,
  Clock,
  Cpu,
  Radio,
  RefreshCw,
  Send,
  ShieldCheck,
  TrendingUp,
  Zap,
} from 'lucide-react';
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchDestinationsRequest } from '../store/slices/destinationsSlice';
import { fetchEventsRequest } from '../store/slices/eventsSlice';
import { fetchOverviewRequest, setPeriod } from '../store/slices/overviewSlice';

interface OverviewPageProps {
  onOpenSendModal: () => void;
}

export const OverviewPage: React.FC<OverviewPageProps> = ({ onOpenSendModal }) => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { overviewStats, systemStats, timeseries, period, isLoading } = useAppSelector(
    (state) => state.overview
  );
  const { events } = useAppSelector((state) => state.events);
  const { destinations } = useAppSelector((state) => state.destinations);

  const refreshAll = () => {
    dispatch(fetchOverviewRequest({ period }));
    dispatch(fetchEventsRequest());
    dispatch(fetchDestinationsRequest());
  };

  useEffect(() => {
    refreshAll();
    const interval = setInterval(refreshAll, 8000);
    return () => clearInterval(interval);
  }, [period]);

  const totalEvents = overviewStats?.total_events ?? systemStats?.total_events ?? 0;
  const successRate = overviewStats?.success_rate ?? 100.0;
  const p50Latency = overviewStats?.p50_latency_ms ?? 34;
  const dlqCount = systemStats?.dead_letter_deliveries ?? 0;

  const chartData = timeseries.length > 0
    ? timeseries.map((pt) => ({
        time: new Date(pt.bucket).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        events: pt.value,
      }))
    : [
        { time: '00:00', events: 12 },
        { time: '04:00', events: 25 },
        { time: '08:00', events: 88 },
        { time: '12:00', events: 145 },
        { time: '16:00', events: 210 },
        { time: '20:00', events: 160 },
        { time: 'Now', events: 195 },
      ];

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-200">
      {/* Top Banner & Quick Action Cards */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-xl font-bold text-white tracking-tight">System Telemetry & Controls</h1>
            <p className="text-xs text-zinc-400">Live webhook ingestion, verification and resilient delivery.</p>
          </div>
          <div className="flex items-center space-x-2">
            <button
              onClick={refreshAll}
              className="flex items-center space-x-1 px-3 py-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-xs text-zinc-300 hover:text-white hover:bg-zinc-800 transition-all"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
              <span>Refresh</span>
            </button>
          </div>
        </div>

        {/* Quick Action Tiles */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div
            onClick={() => navigate('/sources')}
            className="p-5 rounded-2xl bg-gradient-to-b from-zinc-900/90 to-zinc-950/80 border border-zinc-800/80 hover:border-zinc-700 cursor-pointer group transition-all"
          >
            <div className="w-9 h-9 rounded-xl bg-zinc-800/80 border border-zinc-700 flex items-center justify-center mb-3 group-hover:scale-105 transition-transform">
              <Radio className="w-5 h-5 text-emerald-400" />
            </div>
            <h3 className="text-sm font-semibold text-white group-hover:text-emerald-400 transition-colors flex items-center justify-between">
              <span>Create Inbound Source</span>
              <ArrowUpRight className="w-4 h-4 text-zinc-500 group-hover:text-emerald-400" />
            </h3>
            <p className="text-xs text-zinc-400 mt-1">
              Configure Stripe, GitHub, Shopify or generic HMAC webhooks with signature checks.
            </p>
          </div>

          <div
            onClick={() => navigate('/destinations')}
            className="p-5 rounded-2xl bg-gradient-to-b from-zinc-900/90 to-zinc-950/80 border border-zinc-800/80 hover:border-zinc-700 cursor-pointer group transition-all"
          >
            <div className="w-9 h-9 rounded-xl bg-zinc-800/80 border border-zinc-700 flex items-center justify-center mb-3 group-hover:scale-105 transition-transform">
              <Cpu className="w-5 h-5 text-blue-400" />
            </div>
            <h3 className="text-sm font-semibold text-white group-hover:text-blue-400 transition-colors flex items-center justify-between">
              <span>Connect Destination</span>
              <ArrowUpRight className="w-4 h-4 text-zinc-500 group-hover:text-blue-400" />
            </h3>
            <p className="text-xs text-zinc-400 mt-1">
              Add downstream HTTP endpoints, rate limiters, backoff and circuit breaker thresholds.
            </p>
          </div>

          <div
            onClick={onOpenSendModal}
            className="p-5 rounded-2xl bg-gradient-to-b from-zinc-900/90 to-zinc-950/80 border border-zinc-800/80 hover:border-zinc-700 cursor-pointer group transition-all"
          >
            <div className="w-9 h-9 rounded-xl bg-zinc-800/80 border border-zinc-700 flex items-center justify-center mb-3 group-hover:scale-105 transition-transform">
              <Send className="w-5 h-5 text-violet-400" />
            </div>
            <h3 className="text-sm font-semibold text-white group-hover:text-violet-400 transition-colors flex items-center justify-between">
              <span>Dispatch Test Webhook</span>
              <ArrowUpRight className="w-4 h-4 text-zinc-500 group-hover:text-violet-400" />
            </h3>
            <p className="text-xs text-zinc-400 mt-1">
              Send live JSON payloads to `/hooks/:slug` to test verification & routing immediately.
            </p>
          </div>
        </div>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        {/* Total Ingestion */}
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800">
          <div className="flex items-center justify-between text-xs font-mono text-zinc-400">
            <span>TOTAL EVENTS</span>
            <Zap className="w-4 h-4 text-zinc-500" />
          </div>
          <div className="mt-3 text-2xl font-bold font-mono text-white tracking-tight">
            {totalEvents.toLocaleString()}
          </div>
          <div className="mt-1 flex items-center space-x-1.5 text-xs text-emerald-400 font-mono">
            <TrendingUp className="w-3.5 h-3.5" />
            <span>Redux-Saga Synced</span>
          </div>
        </div>

        {/* Success Rate */}
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800">
          <div className="flex items-center justify-between text-xs font-mono text-zinc-400">
            <span>DELIVERY SUCCESS</span>
            <ShieldCheck className="w-4 h-4 text-zinc-500" />
          </div>
          <div className="mt-3 text-2xl font-bold font-mono text-emerald-400 tracking-tight">
            {successRate.toFixed(1)}%
          </div>
          <div className="mt-1 text-xs text-zinc-500 font-mono">
            Across active subscriptions
          </div>
        </div>

        {/* P50 Latency */}
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800">
          <div className="flex items-center justify-between text-xs font-mono text-zinc-400">
            <span>P50 LATENCY</span>
            <Clock className="w-4 h-4 text-zinc-500" />
          </div>
          <div className="mt-3 text-2xl font-bold font-mono text-white tracking-tight">
            {p50Latency ? `${p50Latency.toFixed(0)} ms` : '< 40 ms'}
          </div>
          <div className="mt-1 text-xs text-zinc-500 font-mono">
            End-to-end relay duration
          </div>
        </div>

        {/* DLQ Count */}
        <div
          onClick={() => navigate('/dlq')}
          className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 hover:border-zinc-700 cursor-pointer transition-colors group"
        >
          <div className="flex items-center justify-between text-xs font-mono text-zinc-400">
            <span>DEAD LETTER QUEUE</span>
            <AlertTriangle className={`w-4 h-4 ${dlqCount > 0 ? 'text-amber-400' : 'text-zinc-500'}`} />
          </div>
          <div className={`mt-3 text-2xl font-bold font-mono tracking-tight ${dlqCount > 0 ? 'text-amber-400' : 'text-zinc-200'}`}>
            {dlqCount}
          </div>
          <div className="mt-1 text-xs text-zinc-400 font-mono flex items-center space-x-1 group-hover:text-white transition-colors">
            <span>Quarantined items</span>
            <ArrowUpRight className="w-3 h-3" />
          </div>
        </div>
      </div>

      {/* Chart Section */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-sm font-semibold text-white">Event Ingestion Throughput</h2>
            <p className="text-xs text-zinc-400">Real-time webhook traffic ingested through API gateway.</p>
          </div>
          <div className="flex items-center space-x-1 bg-zinc-950 p-1 rounded-lg border border-zinc-800">
            {(['1h', '24h', '7d', '30d'] as const).map((p) => (
              <button
                key={p}
                onClick={() => dispatch(setPeriod(p))}
                className={`px-3 py-1 text-xs font-mono rounded-md transition-colors ${
                  period === p
                    ? 'bg-zinc-800 text-white font-semibold shadow-sm'
                    : 'text-zinc-400 hover:text-zinc-200'
                }`}
              >
                {p}
              </button>
            ))}
          </div>
        </div>

        <div className="h-64 w-full">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={chartData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
              <defs>
                <linearGradient id="eventGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#10b981" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="#27272a" opacity={0.6} />
              <XAxis dataKey="time" stroke="#71717a" fontSize={11} tickLine={false} />
              <YAxis stroke="#71717a" fontSize={11} tickLine={false} />
              <Tooltip
                contentStyle={{ backgroundColor: '#18181b', borderColor: '#27272a', borderRadius: '8px', fontSize: '12px' }}
                itemStyle={{ color: '#10b981' }}
              />
              <Area type="monotone" dataKey="events" stroke="#10b981" strokeWidth={2} fillOpacity={1} fill="url(#eventGrad)" />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Split Section: Recent Events + Active Destinations */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Recent Events */}
        <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-white flex items-center space-x-2">
              <span>Recent Webhook Events</span>
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
            </h2>
            <button
              onClick={() => navigate('/events')}
              className="text-xs text-zinc-400 hover:text-white font-mono flex items-center space-x-1"
            >
              <span>View all</span>
              <ArrowUpRight className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="space-y-2">
            {events.length === 0 ? (
              <div className="p-8 text-center text-xs text-zinc-500 border border-dashed border-zinc-800 rounded-xl">
                No events received yet. Click "Dispatch Test Webhook" to send your first event!
              </div>
            ) : (
              events.slice(0, 5).map((evt) => (
                <div
                  key={evt.id}
                  onClick={() => navigate('/events')}
                  className="p-3 rounded-xl bg-zinc-900/50 hover:bg-zinc-900 border border-zinc-800/80 hover:border-zinc-700 cursor-pointer transition-colors flex items-center justify-between"
                >
                  <div className="flex items-center space-x-3">
                    <div className="p-2 rounded-lg bg-zinc-800 text-emerald-400">
                      <Zap className="w-3.5 h-3.5" />
                    </div>
                    <div>
                      <div className="text-xs font-mono font-medium text-zinc-200">{evt.event_type}</div>
                      <div className="text-[10px] font-mono text-zinc-500 truncate max-w-[200px]">{evt.id}</div>
                    </div>
                  </div>
                  <div className="text-right">
                    <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">
                      Ingested
                    </span>
                    <div className="text-[10px] font-mono text-zinc-500 mt-0.5">
                      {new Date(evt.created_at).toLocaleTimeString()}
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Active Destinations */}
        <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-white">Destination Endpoints & Circuit Health</h2>
            <button
              onClick={() => navigate('/destinations')}
              className="text-xs text-zinc-400 hover:text-white font-mono flex items-center space-x-1"
            >
              <span>Manage</span>
              <ArrowUpRight className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="space-y-2">
            {destinations.length === 0 ? (
              <div className="p-8 text-center text-xs text-zinc-500 border border-dashed border-zinc-800 rounded-xl">
                No destination endpoints registered. Click "Connect Destination" to add one!
              </div>
            ) : (
              destinations.slice(0, 4).map((dest) => (
                <div
                  key={dest.id}
                  className="p-3 rounded-xl bg-zinc-900/50 border border-zinc-800/80 flex items-center justify-between"
                >
                  <div className="flex items-center space-x-3">
                    <div className="p-2 rounded-lg bg-zinc-800 text-blue-400">
                      <Cpu className="w-3.5 h-3.5" />
                    </div>
                    <div>
                      <div className="text-xs font-semibold text-zinc-200">{dest.name}</div>
                      <div className="text-[11px] font-mono text-zinc-500 truncate max-w-[240px]">{dest.url}</div>
                    </div>
                  </div>
                  <div className="text-right">
                    <span
                      className={`text-[10px] font-mono px-2 py-0.5 rounded-full border font-medium ${
                        dest.circuit_status === 'closed'
                          ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                          : dest.circuit_status === 'half_open'
                          ? 'bg-amber-500/10 text-amber-400 border-amber-500/20'
                          : 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                      }`}
                    >
                      {dest.circuit_status.toUpperCase()}
                    </span>
                    <div className="text-[10px] font-mono text-zinc-500 mt-0.5">
                      Max {dest.max_retry_count} retries
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
