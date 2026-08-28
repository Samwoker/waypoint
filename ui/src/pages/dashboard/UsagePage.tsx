import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import {
  AlertTriangle,
  ArrowRight,
  BarChart3,
  Calendar,
  CheckCircle2,
  Cpu,
  Layers,
  Radio,
  RefreshCw,
  Server,
  ShieldAlert,
  Sparkles,
  Zap,
} from 'lucide-react';
import { useAppSelector } from '../../store/hooks';
import { api } from '../../api/client';
import { DailyEventCount, TenantUsage } from '../../types';
import { getPlan, formatEventLimit, getUsagePercentage } from '../../config/plans';

export const UsagePage: React.FC = () => {
  const { user } = useAppSelector((state) => state.auth);
  const { sources } = useAppSelector((state) => state.sources);
  const { destinations } = useAppSelector((state) => state.destinations);
  const { apiKeys } = useAppSelector((state) => state.apiKeys);

  const [period, setPeriod] = useState<'30d' | '7d' | '24h'>('30d');
  const [usage, setUsage] = useState<TenantUsage | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  useEffect(() => {
    if (user?.tenant_id) {
      setIsLoading(true);
      api
        .getTenantUsage(user.tenant_id, period)
        .then((data) => setUsage(data))
        .catch((err) => console.error('Failed to load tenant usage:', err))
        .finally(() => setIsLoading(false));
    }
  }, [user?.tenant_id, period]);

  const currentPlan = getPlan('free'); // Default to Free tier
  const totalEventsUsed = usage?.total_events || 0;
  const eventUsagePercent = getUsagePercentage(totalEventsUsed, currentPlan.eventLimit);
  const isNearLimit = eventUsagePercent >= 80 && eventUsagePercent < 100;
  const isAtLimit = eventUsagePercent >= 100;

  const activeSourcesCount = sources.length;
  const activeDestinationsCount = destinations.length;
  const activeApiKeysCount = apiKeys.length;

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-200 font-sans">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-zinc-800 pb-6">
        <div>
          <div className="flex items-center space-x-2.5">
            <span className="text-xs font-mono font-semibold uppercase text-emerald-400">
              Subscription Metering
            </span>
            <span className="px-2 py-0.5 rounded-md text-[10px] font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase">
              {currentPlan.name} Tier
            </span>
          </div>
          <h1 className="text-2xl sm:text-3xl font-extrabold text-white tracking-tight mt-1">
            Usage & Plan Limits
          </h1>
          <p className="text-xs sm:text-sm text-zinc-400 mt-1">
            Track monthly event ingestion quotas, target destination limits, and daily volume metrics.
          </p>
        </div>

        {/* Period Selector & Upgrade CTA */}
        <div className="flex items-center space-x-3">
          <div className="flex bg-zinc-950 p-1 rounded-xl border border-zinc-800">
            {(['24h', '7d', '30d'] as const).map((p) => (
              <button
                key={p}
                type="button"
                onClick={() => setPeriod(p)}
                className={`px-3 py-1 text-xs font-mono rounded-lg transition-all ${
                  period === p
                    ? 'bg-zinc-800 text-white font-bold shadow-sm'
                    : 'text-zinc-500 hover:text-zinc-300'
                }`}
              >
                {p}
              </button>
            ))}
          </div>

          <Link
            to="/dashboard/billing"
            className="flex items-center space-x-1.5 px-4 py-2 rounded-xl text-xs font-semibold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-md shadow-emerald-500/10 transition-all"
          >
            <Sparkles className="w-3.5 h-3.5" />
            <span>Upgrade Plan</span>
          </Link>
        </div>
      </div>

      {/* Warning Banners at 80% / 100% */}
      {isAtLimit ? (
        <div className="p-4 rounded-2xl bg-rose-950/40 border border-rose-800/80 text-rose-200 flex items-start space-x-3 text-xs leading-relaxed animate-in fade-in">
          <ShieldAlert className="w-5 h-5 text-rose-400 shrink-0 mt-0.5" />
          <div className="flex-1 space-y-1">
            <h4 className="font-bold uppercase tracking-wider text-[11px] font-mono">
              Monthly Event Allowance Reached (100%)
            </h4>
            <p>
              Your organization has ingested {totalEventsUsed.toLocaleString()} of {currentPlan.eventLimit.toLocaleString()} monthly events allowed on the {currentPlan.name} plan. Upgrade now to ensure uninterrupted webhook deliveries.
            </p>
          </div>
          <Link
            to="/dashboard/billing"
            className="px-3 py-1.5 rounded-lg bg-rose-500 text-zinc-950 font-bold text-xs shrink-0 hover:bg-rose-400 transition-colors"
          >
            Upgrade Now
          </Link>
        </div>
      ) : isNearLimit ? (
        <div className="p-4 rounded-2xl bg-amber-950/40 border border-amber-800/80 text-amber-200 flex items-start space-x-3 text-xs leading-relaxed animate-in fade-in">
          <AlertTriangle className="w-5 h-5 text-amber-400 shrink-0 mt-0.5" />
          <div className="flex-1 space-y-1">
            <h4 className="font-bold uppercase tracking-wider text-[11px] font-mono">
              Approaching Monthly Allowance ({eventUsagePercent}%)
            </h4>
            <p>
              You have consumed {totalEventsUsed.toLocaleString()} of your {currentPlan.eventLimit.toLocaleString()} monthly events. Consider upgrading before reaching the limit.
            </p>
          </div>
          <Link
            to="/dashboard/billing"
            className="px-3 py-1.5 rounded-lg bg-amber-500 text-zinc-950 font-bold text-xs shrink-0 hover:bg-amber-400 transition-colors"
          >
            View Plans
          </Link>
        </div>
      ) : null}

      {/* Main Quotas 4-Card Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Card 1: Monthly Events */}
        <div className="p-5 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-3 shadow-sm">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono font-medium text-zinc-400 uppercase">
              Monthly Ingested Events
            </span>
            <div className="w-7 h-7 rounded-lg bg-emerald-500/10 text-emerald-400 flex items-center justify-center">
              <Zap className="w-4 h-4" />
            </div>
          </div>

          <div className="space-y-1">
            <div className="flex items-baseline justify-between">
              <span className="text-2xl font-extrabold text-white">
                {totalEventsUsed.toLocaleString()}
              </span>
              <span className="text-xs font-mono text-zinc-500">
                / {formatEventLimit(currentPlan.eventLimit)}
              </span>
            </div>

            {/* Progress Bar */}
            <div className="w-full h-2 bg-zinc-950 rounded-full overflow-hidden border border-zinc-800">
              <div
                className={`h-full transition-all duration-500 ${
                  isAtLimit
                    ? 'bg-rose-500'
                    : isNearLimit
                    ? 'bg-amber-500'
                    : 'bg-emerald-500'
                }`}
                style={{ width: `${eventUsagePercent}%` }}
              />
            </div>
          </div>

          <div className="text-[11px] font-mono text-zinc-500 flex justify-between">
            <span>{eventUsagePercent}% utilized</span>
            <span>{Math.max(0, currentPlan.eventLimit - totalEventsUsed).toLocaleString()} remaining</span>
          </div>
        </div>

        {/* Card 2: Target Destinations */}
        <div className="p-5 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-3 shadow-sm">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono font-medium text-zinc-400 uppercase">
              Destinations
            </span>
            <div className="w-7 h-7 rounded-lg bg-blue-500/10 text-blue-400 flex items-center justify-center">
              <Radio className="w-4 h-4" />
            </div>
          </div>

          <div className="space-y-1">
            <div className="flex items-baseline justify-between">
              <span className="text-2xl font-extrabold text-white">
                {activeDestinationsCount}
              </span>
              <span className="text-xs font-mono text-zinc-500">
                / {currentPlan.destinationLimit}
              </span>
            </div>

            <div className="w-full h-2 bg-zinc-950 rounded-full overflow-hidden border border-zinc-800">
              <div
                className="h-full bg-blue-500 transition-all duration-500"
                style={{
                  width: `${Math.min(100, (activeDestinationsCount / currentPlan.destinationLimit) * 100)}%`,
                }}
              />
            </div>
          </div>

          <div className="text-[11px] font-mono text-zinc-500 flex justify-between">
            <span>{Math.max(0, currentPlan.destinationLimit - activeDestinationsCount)} available</span>
            <span>{currentPlan.destinationLimit} Max</span>
          </div>
        </div>

        {/* Card 3: Inbound Sources */}
        <div className="p-5 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-3 shadow-sm">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono font-medium text-zinc-400 uppercase">
              Inbound Sources
            </span>
            <div className="w-7 h-7 rounded-lg bg-purple-500/10 text-purple-400 flex items-center justify-center">
              <Layers className="w-4 h-4" />
            </div>
          </div>

          <div className="space-y-1">
            <div className="flex items-baseline justify-between">
              <span className="text-2xl font-extrabold text-white">
                {activeSourcesCount}
              </span>
              <span className="text-xs font-mono text-zinc-500">
                / {currentPlan.sourceLimit}
              </span>
            </div>

            <div className="w-full h-2 bg-zinc-950 rounded-full overflow-hidden border border-zinc-800">
              <div
                className="h-full bg-purple-500 transition-all duration-500"
                style={{
                  width: `${Math.min(100, (activeSourcesCount / currentPlan.sourceLimit) * 100)}%`,
                }}
              />
            </div>
          </div>

          <div className="text-[11px] font-mono text-zinc-500 flex justify-between">
            <span>{Math.max(0, currentPlan.sourceLimit - activeSourcesCount)} available</span>
            <span>{currentPlan.sourceLimit} Max</span>
          </div>
        </div>

        {/* Card 4: Scoped API Keys */}
        <div className="p-5 rounded-2xl bg-[#0e0e11] border border-zinc-800 space-y-3 shadow-sm">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono font-medium text-zinc-400 uppercase">
              API Keys
            </span>
            <div className="w-7 h-7 rounded-lg bg-amber-500/10 text-amber-400 flex items-center justify-center">
              <Server className="w-4 h-4" />
            </div>
          </div>

          <div className="space-y-1">
            <div className="flex items-baseline justify-between">
              <span className="text-2xl font-extrabold text-white">
                {activeApiKeysCount}
              </span>
              <span className="text-xs font-mono text-zinc-500">
                / {currentPlan.apiKeyLimit}
              </span>
            </div>

            <div className="w-full h-2 bg-zinc-950 rounded-full overflow-hidden border border-zinc-800">
              <div
                className="h-full bg-amber-500 transition-all duration-500"
                style={{
                  width: `${Math.min(100, (activeApiKeysCount / currentPlan.apiKeyLimit) * 100)}%`,
                }}
              />
            </div>
          </div>

          <div className="text-[11px] font-mono text-zinc-500 flex justify-between">
            <span>{Math.max(0, currentPlan.apiKeyLimit - activeApiKeysCount)} available</span>
            <span>{currentPlan.apiKeyLimit} Max</span>
          </div>
        </div>
      </div>

      {/* Daily Volume Breakdown Table / Chart */}
      <div className="p-6 rounded-3xl bg-[#0e0e11] border border-zinc-800 space-y-4 shadow-sm">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <BarChart3 className="w-4 h-4 text-emerald-400" />
            <h3 className="text-sm font-bold text-white">Daily Ingestion & Delivery Volume</h3>
          </div>
          <span className="text-xs font-mono text-zinc-500">
            Total Outbound Delivery Attempts:{' '}
            <strong className="text-zinc-200 font-bold">
              {usage?.total_delivery_attempts?.toLocaleString() || 0}
            </strong>
          </span>
        </div>

        {usage?.daily_events && usage.daily_events.length > 0 ? (
          <div className="overflow-x-auto rounded-2xl border border-zinc-800/80">
            <table className="w-full text-left text-xs">
              <thead>
                <tr className="border-b border-zinc-800 bg-zinc-950/80 font-mono text-[10px] uppercase text-zinc-500">
                  <th className="py-3 px-4 font-semibold">Date</th>
                  <th className="py-3 px-4 font-semibold">Events Ingested</th>
                  <th className="py-3 px-4 font-semibold">Volume Distribution</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/50">
                {usage.daily_events.map((d: DailyEventCount, idx: number) => {
                  const maxDayCount = Math.max(
                    ...usage.daily_events.map((e: DailyEventCount) => Number(e.count) || 1)
                  );
                  const percent = Math.min(100, Math.round((Number(d.count) / maxDayCount) * 100));

                  return (
                    <tr key={idx} className="hover:bg-zinc-900/40 font-mono text-zinc-300">
                      <td className="py-2.5 px-4 text-zinc-400">{d.date}</td>
                      <td className="py-2.5 px-4 font-bold text-white">
                        {Number(d.count).toLocaleString()}
                      </td>
                      <td className="py-2.5 px-4 w-1/2">
                        <div className="w-full h-1.5 bg-zinc-950 rounded-full overflow-hidden">
                          <div
                            className="h-full bg-emerald-500 rounded-full"
                            style={{ width: `${percent}%` }}
                          />
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="py-8 text-center text-xs text-zinc-500 font-mono">
            {isLoading ? 'Loading usage metrics...' : `No event data recorded for the selected ${period} period.`}
          </div>
        )}
      </div>
    </div>
  );
};
