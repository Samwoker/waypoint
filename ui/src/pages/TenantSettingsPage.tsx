import React, { useEffect, useState } from 'react';
import {
  Building,
  Shield,
  Layers,
  Send,
  Calendar,
  Key,
  TrendingUp,
  Clock,
  CheckCircle2,
  Users,
  Copy,
  Check,
} from 'lucide-react';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { api } from '../api/client';
import { TenantUsage, Tenant } from '../types';
import { useAppSelector } from '../store/hooks';
import { useToast } from '../context/ToastContext';
import { Skeleton } from '../components/common/Skeleton';

export const TenantSettingsPage: React.FC = () => {
  const toast = useToast();
  const { currentTenant, user } = useAppSelector((state) => state.auth);
  const [usage, setUsage] = useState<TenantUsage | null>(null);
  const [period, setPeriod] = useState<string>('30d');
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [copiedId, setCopiedId] = useState<boolean>(false);

  const fetchUsage = async () => {
    if (!currentTenant?.id) return;
    try {
      setIsLoading(true);
      const data = await api.getTenantUsage(currentTenant.id, period);
      setUsage(data);
    } catch (err: any) {
      toast.error('Failed to load tenant usage', err.message);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchUsage();
  }, [currentTenant?.id, period]);

  const handleCopyId = () => {
    if (!currentTenant?.id) return;
    navigator.clipboard.writeText(currentTenant.id);
    setCopiedId(true);
    toast.success('Tenant UUID copied');
    setTimeout(() => setCopiedId(false), 2000);
  };

  if (isLoading && !usage) {
    return (
      <div className="p-8 max-w-7xl mx-auto space-y-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-40 w-full rounded-2xl" />
        <Skeleton className="h-64 w-full rounded-2xl" />
      </div>
    );
  }

  const quotaLimit = 100000;
  const eventsCount = usage?.total_events || 0;
  const quotaPercent = Math.min(100, (eventsCount / quotaLimit) * 100);

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-extrabold text-white tracking-tight">
          Organization & Tenant Settings
        </h1>
        <p className="text-xs text-zinc-400 mt-1">
          Manage workspace profile, quota utilization, and team members.
        </p>
      </div>

      {/* Workspace Profile Card */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-5">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div className="flex items-center space-x-3.5">
            <div className="p-3 rounded-2xl bg-zinc-900 border border-zinc-800 text-white">
              <Building className="w-6 h-6" />
            </div>
            <div>
              <div className="flex items-center space-x-2">
                <h2 className="text-lg font-bold text-white tracking-tight">
                  {currentTenant?.name || 'Production Workspace'}
                </h2>
                <span className="px-2 py-0.5 rounded-full text-[10px] font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-semibold">
                  Enterprise Plan
                </span>
              </div>
              <div className="text-xs text-zinc-400 font-mono">
                Slug: {currentTenant?.slug || 'default'}
              </div>
            </div>
          </div>

          <div className="flex items-center space-x-2 bg-zinc-950 px-3 py-1.5 rounded-xl border border-zinc-800">
            <span className="text-[10px] font-mono text-zinc-500">Tenant UUID:</span>
            <code className="text-xs font-mono text-zinc-300">
              {currentTenant?.id ? `${currentTenant.id.slice(0, 16)}...` : 'default'}
            </code>
            <button
              onClick={handleCopyId}
              className="p-1 text-zinc-500 hover:text-white transition-colors"
            >
              {copiedId ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
            </button>
          </div>
        </div>
      </div>

      {/* Quota & Ingestion Usage Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-3">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">Monthly Ingestion Quota</span>
            <Layers className="w-4 h-4 text-purple-400" />
          </div>

          <div className="space-y-1.5">
            <div className="flex justify-between text-xs font-mono">
              <span className="text-white font-bold">{eventsCount.toLocaleString()}</span>
              <span className="text-zinc-500">/ {quotaLimit.toLocaleString()} events</span>
            </div>
            <div className="w-full h-2 rounded-full bg-zinc-900 overflow-hidden">
              <div
                className="h-full bg-emerald-400 transition-all"
                style={{ width: `${quotaPercent}%` }}
              />
            </div>
          </div>
          <div className="text-[10px] font-mono text-zinc-500">
            {(100 - quotaPercent).toFixed(1)}% remaining this billing cycle
          </div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">Total Ingested Events</span>
            <Calendar className="w-4 h-4 text-blue-400" />
          </div>
          <div className="text-2xl font-black text-white font-mono pt-1">
            {usage?.total_events.toLocaleString() || '0'}
          </div>
          <div className="text-[10px] font-mono text-zinc-500">Inbound Webhooks Processed</div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span className="font-mono text-[10px] uppercase">Delivery Attempts</span>
            <Send className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-2xl font-black text-white font-mono pt-1">
            {usage?.total_delivery_attempts.toLocaleString() || '0'}
          </div>
          <div className="text-[10px] font-mono text-zinc-500">Outbound Forwarding Dispatches</div>
        </div>
      </div>

      {/* Daily Event Ingestion Breakdown Chart */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h3 className="text-sm font-bold text-white tracking-tight">
              Daily Webhook Ingestion Volume
            </h3>
            <p className="text-xs text-zinc-400">
              Aggregated daily event counts for tenant over the last {period}.
            </p>
          </div>
        </div>

        <div className="h-64 w-full pt-4">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={usage?.daily_events || []}>
              <CartesianGrid strokeDasharray="3 3" stroke="#27272a" vertical={false} />
              <XAxis dataKey="date" stroke="#71717a" fontSize={10} />
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
              <Bar dataKey="count" name="Daily Events" fill="#34d399" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
};
