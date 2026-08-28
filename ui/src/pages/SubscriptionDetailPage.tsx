import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  Zap,
  ArrowLeft,
  Pause,
  Play,
  Trash2,
  Radio,
  Send,
  ArrowRight,
  Code2,
  Activity,
  CheckCircle2,
  Clock,
  XCircle,
} from 'lucide-react';
import { api } from '../api/client';
import { Subscription, Delivery, Source, Destination } from '../types';
import { useToast } from '../context/ToastContext';
import { ConfirmModal } from '../components/common/ConfirmModal';
import { Skeleton } from '../components/common/Skeleton';

export const SubscriptionDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const toast = useToast();

  const [subscription, setSubscription] = useState<Subscription | null>(null);
  const [source, setSource] = useState<Source | null>(null);
  const [destination, setDestination] = useState<Destination | null>(null);
  const [deliveries, setDeliveries] = useState<Delivery[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const [isToggling, setIsToggling] = useState<boolean>(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState<boolean>(false);
  const [isDeleting, setIsDeleting] = useState<boolean>(false);

  const fetchSubscriptionData = async () => {
    if (!id) return;
    try {
      setIsLoading(true);
      const [sub, allDels] = await Promise.all([
        api.getSubscription(id),
        api.listDeliveries(undefined, 20),
      ]);
      setSubscription(sub);
      setDeliveries(allDels.deliveries.filter((d) => d.subscription_id === id));

      if (sub.source_id) {
        try {
          const src = await api.getSource(sub.source_id);
          setSource(src);
        } catch (_) {}
      }
      if (sub.destination_id) {
        try {
          const dest = await api.getDestination(sub.destination_id);
          setDestination(dest);
        } catch (_) {}
      }
    } catch (err: any) {
      toast.error('Failed to load subscription details', err.message);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchSubscriptionData();
  }, [id]);

  const handleToggle = async () => {
    if (!id || !subscription) return;
    try {
      setIsToggling(true);
      if (subscription.is_active) {
        await api.pauseSubscription(id);
        toast.warning('Subscription Paused', 'Incoming events matching these rules will not be queued for delivery.');
      } else {
        await api.resumeSubscription(id);
        toast.success('Subscription Resumed', 'Routing rule is active and forwarding matching webhooks.');
      }
      fetchSubscriptionData();
    } catch (err: any) {
      toast.error('Failed to update subscription status', err.message);
    } finally {
      setIsToggling(false);
    }
  };

  const handleDelete = async () => {
    if (!id) return;
    try {
      setIsDeleting(true);
      await api.deleteSubscription(id);
      setIsDeleteModalOpen(false);
      toast.success('Subscription deleted successfully');
      navigate('/subscriptions');
    } catch (err: any) {
      toast.error('Failed to delete subscription', err.message);
    } finally {
      setIsDeleting(false);
    }
  };

  if (isLoading) {
    return (
      <div className="p-8 max-w-7xl mx-auto space-y-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-40 w-full rounded-2xl" />
        <Skeleton className="h-64 w-full rounded-2xl" />
      </div>
    );
  }

  if (!subscription) {
    return (
      <div className="p-8 max-w-7xl mx-auto text-center space-y-4">
        <p className="text-sm text-zinc-400">Subscription routing rule not found.</p>
        <button
          onClick={() => navigate('/subscriptions')}
          className="px-4 py-2 text-xs font-semibold rounded-xl bg-zinc-800 text-white"
        >
          Back to Subscriptions
        </button>
      </div>
    );
  }

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Top Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-center space-x-3">
          <button
            onClick={() => navigate('/subscriptions')}
            className="p-2 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-white border border-zinc-800 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div className="space-y-0.5">
            <div className="flex items-center space-x-2.5">
              <h1 className="text-xl font-bold text-white tracking-tight">Routing Subscription</h1>
              <span
                className={`px-2 py-0.5 rounded-full text-[10px] font-mono font-semibold uppercase ${
                  subscription.is_active
                    ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                    : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                }`}
              >
                {subscription.is_active ? 'Active' : 'Paused'}
              </span>
            </div>
            <p className="text-xs text-zinc-400 font-mono">ID: {subscription.id}</p>
          </div>
        </div>

        <div className="flex items-center space-x-2.5">
          <button
            onClick={handleToggle}
            disabled={isToggling}
            className={`px-3.5 py-2 rounded-xl border text-xs font-semibold flex items-center space-x-1.5 transition-colors disabled:opacity-50 ${
              subscription.is_active
                ? 'bg-amber-950/40 hover:bg-amber-900/50 border-amber-800/40 text-amber-300'
                : 'bg-emerald-950/40 hover:bg-emerald-900/50 border-emerald-800/40 text-emerald-300'
            }`}
          >
            {subscription.is_active ? (
              <>
                <Pause className="w-3.5 h-3.5" />
                <span>Pause Routing</span>
              </>
            ) : (
              <>
                <Play className="w-3.5 h-3.5" />
                <span>Resume Routing</span>
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

      {/* Visual Source -> Subscription -> Destination Pipe */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <span className="text-[11px] font-mono font-bold uppercase tracking-wider text-zinc-400">
          Connected Pipeline Topology
        </span>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 items-center pt-2">
          {/* Source Endpoint Card */}
          <div
            onClick={() => navigate(`/sources/${subscription.source_id}`)}
            className="p-4 rounded-xl bg-zinc-950 hover:bg-zinc-900/70 border border-zinc-800 cursor-pointer transition-all space-y-2 group"
          >
            <div className="flex items-center space-x-2 text-zinc-400 text-[10px] font-mono uppercase">
              <Radio className="w-3.5 h-3.5 text-blue-400" />
              <span>Inbound Source</span>
            </div>
            <div className="text-sm font-bold text-white group-hover:text-emerald-400 font-mono transition-colors">
              {source?.name || subscription.source_name || 'Source Endpoint'}
            </div>
            <div className="text-[11px] text-zinc-500 font-mono">
              /hooks/{source?.slug || 'slug'}
            </div>
          </div>

          {/* Subscription Filter Middle Node */}
          <div className="p-4 rounded-xl bg-zinc-900/90 border border-purple-500/20 space-y-2 text-center">
            <div className="flex items-center justify-center space-x-1.5 text-purple-300 text-[10px] font-mono uppercase font-bold">
              <Zap className="w-3.5 h-3.5" />
              <span>Matching Event Filter</span>
            </div>
            <div className="flex flex-wrap justify-center gap-1.5">
              {subscription.event_types.map((et) => (
                <span
                  key={et}
                  className="px-2 py-0.5 rounded bg-purple-950/60 border border-purple-800/60 text-purple-200 text-xs font-mono font-semibold"
                >
                  {et}
                </span>
              ))}
            </div>
          </div>

          {/* Destination Endpoint Card */}
          <div
            onClick={() => navigate(`/destinations/${subscription.destination_id}`)}
            className="p-4 rounded-xl bg-zinc-950 hover:bg-zinc-900/70 border border-zinc-800 cursor-pointer transition-all space-y-2 group"
          >
            <div className="flex items-center space-x-2 text-zinc-400 text-[10px] font-mono uppercase">
              <Send className="w-3.5 h-3.5 text-emerald-400" />
              <span>Target Destination</span>
            </div>
            <div className="text-sm font-bold text-white group-hover:text-emerald-400 font-mono transition-colors">
              {destination?.name || subscription.destination_name || 'Destination Endpoint'}
            </div>
            <div className="text-[11px] text-zinc-500 font-mono truncate">
              {destination?.url || 'https://...'}
            </div>
          </div>
        </div>
      </div>

      {/* Deliveries Generated By This Subscription */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-bold text-white tracking-tight">
            Recent Forwarded Deliveries
          </h3>
          <span className="text-xs font-mono text-zinc-500">
            {deliveries.length} recent deliveries
          </span>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                <th className="py-2.5 px-3">Delivery ID</th>
                <th className="py-2.5 px-3">Status</th>
                <th className="py-2.5 px-3">Attempts</th>
                <th className="py-2.5 px-3">Dispatched At</th>
                <th className="py-2.5 px-3 text-right">Inspect</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 font-mono">
              {deliveries.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-6 text-center text-zinc-500 italic">
                    No webhook deliveries generated by this subscription rule yet.
                  </td>
                </tr>
              ) : (
                deliveries.map((del) => (
                  <tr
                    key={del.id}
                    onClick={() => navigate(`/deliveries/${del.id}`)}
                    className="hover:bg-zinc-900/40 cursor-pointer transition-colors"
                  >
                    <td className="py-2.5 px-3 text-zinc-300 font-bold">
                      {del.id.slice(0, 13)}...
                    </td>
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
        title="Delete Routing Subscription?"
        description="Deleting this subscription will remove the forwarding connection between the Source and Destination. Historical deliveries will remain preserved."
        confirmText="Delete Subscription"
        variant="danger"
        isLoading={isDeleting}
      />
    </div>
  );
};
