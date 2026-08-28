import React, { useEffect, useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import {
  Send,
  ArrowLeft,
  Pause,
  Play,
  Zap,
  Activity,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  Clock,
  Trash2,
  RotateCcw,
  Loader2,
  ExternalLink,
  ShieldAlert,
} from 'lucide-react';
import { api } from '../api/client';
import { Destination, DestinationHealth, Delivery, TestDestinationResponse } from '../types';
import { useToast } from '../context/ToastContext';
import { ConfirmModal } from '../components/common/ConfirmModal';
import { Skeleton } from '../components/common/Skeleton';

export const DestinationDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const toast = useToast();

  const [destination, setDestination] = useState<Destination | null>(null);
  const [health, setHealth] = useState<DestinationHealth | null>(null);
  const [deliveries, setDeliveries] = useState<Delivery[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const [isTesting, setIsTesting] = useState<boolean>(false);
  const [testResult, setTestResult] = useState<TestDestinationResponse | null>(null);
  const [isTogglingStatus, setIsTogglingStatus] = useState<boolean>(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState<boolean>(false);
  const [isDeleting, setIsDeleting] = useState<boolean>(false);

  const fetchDestinationData = async () => {
    if (!id) return;
    try {
      setIsLoading(true);
      const [dest, hlt, allDels] = await Promise.all([
        api.getDestination(id),
        api.getDestinationHealth(id),
        api.listDeliveries(undefined, 20),
      ]);
      setDestination(dest);
      setHealth(hlt);
      setDeliveries(allDels.deliveries.filter((d) => d.destination_id === id));
    } catch (err: any) {
      toast.error('Failed to load destination details', err.message);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchDestinationData();
  }, [id]);

  const handleTestDestination = async () => {
    if (!id) return;
    try {
      setIsTesting(true);
      const res = await api.testDestination(id);
      setTestResult(res);
      if (res.success) {
        toast.success(
          'Destination Test Succeeded',
          `HTTP ${res.http_status || 200} OK in ${res.latency_ms}ms`
        );
      } else {
        toast.error('Destination Test Failed', res.error || `HTTP ${res.http_status}`);
      }
    } catch (err: any) {
      toast.error('Test request error', err.message);
    } finally {
      setIsTesting(false);
    }
  };

  const handleToggleStatus = async () => {
    if (!id || !destination) return;
    try {
      setIsTogglingStatus(true);
      if (destination.is_active) {
        await api.pauseDestination(id);
        toast.warning('Destination Paused', 'Outbound delivery attempts are temporarily held.');
      } else {
        await api.resumeDestination(id);
        toast.success('Destination Resumed', 'Outbound delivery worker is actively forwarding.');
      }
      fetchDestinationData();
    } catch (err: any) {
      toast.error('Failed to update destination state', err.message);
    } finally {
      setIsTogglingStatus(false);
    }
  };

  const handleDelete = async () => {
    if (!id) return;
    try {
      setIsDeleting(true);
      await api.deleteDestination(id);
      setIsDeleteModalOpen(false);
      toast.success('Destination deleted successfully');
      navigate('/destinations');
    } catch (err: any) {
      toast.error('Failed to delete destination', err.message);
    } finally {
      setIsDeleting(false);
    }
  };

  if (isLoading) {
    return (
      <div className="p-8 max-w-7xl mx-auto space-y-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-36 w-full rounded-2xl" />
        <Skeleton className="h-64 w-full rounded-2xl" />
      </div>
    );
  }

  if (!destination) {
    return (
      <div className="p-8 max-w-7xl mx-auto text-center space-y-4">
        <p className="text-sm text-zinc-400">Destination endpoint not found.</p>
        <button
          onClick={() => navigate('/destinations')}
          className="px-4 py-2 text-xs font-semibold rounded-xl bg-zinc-800 text-white"
        >
          Back to Destinations
        </button>
      </div>
    );
  }

  const isCircuitOpen =
    destination.status === 'circuit_open' || (health && health.status === 'circuit_open');

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Top Bar Navigation & Actions */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-center space-x-3">
          <button
            onClick={() => navigate('/destinations')}
            className="p-2 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-white border border-zinc-800 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div className="space-y-0.5">
            <div className="flex items-center space-x-2.5">
              <h1 className="text-xl font-bold text-white tracking-tight">{destination.name}</h1>
              <span
                className={`px-2 py-0.5 rounded-full text-[10px] font-mono font-semibold uppercase ${
                  isCircuitOpen
                    ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20 animate-pulse'
                    : destination.is_active
                    ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                    : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                }`}
              >
                {isCircuitOpen ? 'Circuit Open' : destination.is_active ? 'Active' : 'Paused'}
              </span>
            </div>
            <code className="text-xs text-zinc-400 font-mono select-all">{destination.url}</code>
          </div>
        </div>

        <div className="flex items-center space-x-2.5">
          <button
            onClick={handleTestDestination}
            disabled={isTesting}
            className="px-3.5 py-2 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-200 hover:text-white text-xs font-semibold flex items-center space-x-1.5 transition-colors disabled:opacity-50"
          >
            {isTesting ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <Zap className="w-3.5 h-3.5 text-amber-400" />
            )}
            <span>Test Endpoint</span>
          </button>

          <button
            onClick={handleToggleStatus}
            disabled={isTogglingStatus}
            className={`px-3.5 py-2 rounded-xl border text-xs font-semibold flex items-center space-x-1.5 transition-colors disabled:opacity-50 ${
              destination.is_active
                ? 'bg-amber-950/40 hover:bg-amber-900/50 border-amber-800/40 text-amber-300'
                : 'bg-emerald-950/40 hover:bg-emerald-900/50 border-emerald-800/40 text-emerald-300'
            }`}
          >
            {destination.is_active ? (
              <>
                <Pause className="w-3.5 h-3.5" />
                <span>Pause</span>
              </>
            ) : (
              <>
                <Play className="w-3.5 h-3.5" />
                <span>Resume</span>
              </>
            )}
          </button>

          <button
            onClick={() => setIsDeleteModalOpen(true)}
            className="px-3 py-2 rounded-xl bg-rose-950/40 hover:bg-rose-900/50 border border-rose-800/40 text-rose-300 text-xs font-semibold flex items-center space-x-1.5 transition-colors"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>Delete</span>
          </button>
        </div>
      </div>

      {/* Test Result Live Banner */}
      {testResult && (
        <div
          className={`p-4 rounded-2xl border text-xs flex items-center justify-between animate-in fade-in ${
            testResult.success
              ? 'bg-emerald-950/30 border-emerald-800/40 text-emerald-300'
              : 'bg-rose-950/30 border-rose-800/40 text-rose-300'
          }`}
        >
          <div className="flex items-center space-x-2.5">
            {testResult.success ? (
              <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            ) : (
              <XCircle className="w-4 h-4 text-rose-400 shrink-0" />
            )}
            <span className="font-semibold">
              {testResult.success
                ? `Test Endpoint Responded (HTTP ${testResult.http_status})`
                : `Test Request Failed: ${testResult.error || `HTTP ${testResult.http_status}`}`}
            </span>
          </div>
          <span className="font-mono text-[11px] text-zinc-400">Latency: {testResult.latency_ms}ms</span>
        </div>
      )}

      {/* Operational Health Matrix */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <span className="text-[10px] font-mono text-zinc-500 uppercase tracking-wider">
            Circuit Breaker
          </span>
          <div className="flex items-center space-x-2 pt-1">
            <span
              className={`w-2.5 h-2.5 rounded-full ${
                isCircuitOpen ? 'bg-rose-400 animate-ping' : 'bg-emerald-400'
              }`}
            />
            <span className="text-base font-bold text-white font-mono capitalize">
              {health?.status || 'Closed (Healthy)'}
            </span>
          </div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <span className="text-[10px] font-mono text-zinc-500 uppercase tracking-wider">
            Delivery Success Rate
          </span>
          <div className="text-xl font-extrabold text-white font-mono pt-1">
            {health?.success_rate ? `${(health.success_rate * 100).toFixed(1)}%` : '100.0%'}
          </div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <span className="text-[10px] font-mono text-zinc-500 uppercase tracking-wider">
            Consecutive Failures
          </span>
          <div
            className={`text-xl font-extrabold font-mono pt-1 ${
              (health?.consecutive_failures || 0) > 0 ? 'text-rose-400' : 'text-zinc-200'
            }`}
          >
            {health?.consecutive_failures || 0}
          </div>
        </div>

        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1">
          <span className="text-[10px] font-mono text-zinc-500 uppercase tracking-wider">
            Retry Policy
          </span>
          <div className="text-sm font-semibold text-white font-mono pt-1">
            {destination.max_retries} retries ({destination.timeout_ms}ms timeout)
          </div>
        </div>
      </div>

      {/* Recent Deliveries to this Destination */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-bold text-white tracking-tight">Recent Forwarded Deliveries</h3>
          <span className="text-xs font-mono text-zinc-500">
            Showing last {deliveries.length} attempts
          </span>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                <th className="py-2.5 px-3">Delivery ID</th>
                <th className="py-2.5 px-3">Status</th>
                <th className="py-2.5 px-3">Attempts</th>
                <th className="py-2.5 px-3">Last Dispatched</th>
                <th className="py-2.5 px-3 text-right">Inspect</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 font-mono">
              {deliveries.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-6 text-center text-zinc-500 italic">
                    No webhook deliveries dispatched to this destination yet.
                  </td>
                </tr>
              ) : (
                deliveries.map((del) => (
                  <tr
                    key={del.id}
                    onClick={() => navigate(`/deliveries/${del.id}`)}
                    className="hover:bg-zinc-900/40 cursor-pointer transition-colors"
                  >
                    <td className="py-2.5 px-3 text-zinc-300 font-bold">{del.id.slice(0, 13)}...</td>
                    <td className="py-2.5 px-3">
                      <span
                        className={`px-2 py-0.5 rounded-full text-[10px] font-semibold capitalize ${
                          del.status === 'delivered'
                            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                            : del.status === 'failed' || del.status === 'dead_letter'
                            ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                            : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                        }`}
                      >
                        {del.status.replace('_', ' ')}
                      </span>
                    </td>
                    <td className="py-2.5 px-3 text-zinc-400">
                      {del.attempt_count} / {del.max_attempts}
                    </td>
                    <td className="py-2.5 px-3 text-zinc-400">
                      {new Date(del.created_at).toLocaleTimeString()}
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

      {/* Delete Confirmation Modal */}
      <ConfirmModal
        isOpen={isDeleteModalOpen}
        onClose={() => setIsDeleteModalOpen(false)}
        onConfirm={handleDelete}
        title="Delete Outbound Destination?"
        description="This will permanently delete this target endpoint and remove all associated active routing subscriptions. Forwarding to this URL will cease immediately."
        confirmText="Delete Destination"
        variant="danger"
        isLoading={isDeleting}
      />
    </div>
  );
};
