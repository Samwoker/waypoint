import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  AlertTriangle,
  RotateCcw,
  Trash2,
  Search,
  ExternalLink,
  CheckCircle2,
  RefreshCw,
  Loader2,
  ArrowRight,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchDlqRequest } from '../store/slices/dlqSlice';
import { api } from '../api/client';
import { DlqRecord } from '../types';
import { useToast } from '../context/ToastContext';
import { ConfirmModal } from '../components/common/ConfirmModal';
import { EmptyState, TableSkeleton } from '../components/common/Skeleton';

export const DlqPage: React.FC = () => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const toast = useToast();
  const { items, isLoading } = useAppSelector((state) => state.dlq);

  const [searchQuery, setSearchQuery] = useState('');
  const [discardTargetId, setDiscardTargetId] = useState<string | null>(null);
  const [isDiscarding, setIsDiscarding] = useState<boolean>(false);
  const [isRetryingAll, setIsRetryingAll] = useState<boolean>(false);

  useEffect(() => {
    dispatch(fetchDlqRequest());
  }, [dispatch]);

  const handleRequeue = async (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    try {
      await api.requeueDlqItem(id);
      toast.success('Delivery Requeued', 'Item returned to active worker pipeline.');
      dispatch(fetchDlqRequest());
    } catch (err: any) {
      toast.error('Requeue failed', err.message);
    }
  };

  const handleConfirmDiscard = async () => {
    if (!discardTargetId) return;
    try {
      setIsDiscarding(true);
      await api.discardDlqItem(discardTargetId);
      toast.info('Delivery Discarded', 'Delivery marked as permanently discarded. Event remains preserved.');
      setDiscardTargetId(null);
      dispatch(fetchDlqRequest());
    } catch (err: any) {
      toast.error('Discard failed', err.message);
    } finally {
      setIsDiscarding(false);
    }
  };

  const handleRetryAll = async () => {
    try {
      setIsRetryingAll(true);
      const res = await api.retryAllDlq();
      toast.success(
        'Bulk DLQ Re-Enqueued',
        `Queued ${res.replayed_count || items.length} deliveries for immediate retry.`
      );
      dispatch(fetchDlqRequest());
    } catch (err: any) {
      toast.error('Retry all failed', err.message);
    } finally {
      setIsRetryingAll(false);
    }
  };

  const filteredItems = items.filter(
    (item) =>
      item.delivery_id.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.destination_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.event_type.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center space-x-2.5">
            <h1 className="text-2xl font-extrabold text-white tracking-tight">
              Dead Letter Queue (DLQ)
            </h1>
            {items.length > 0 && (
              <span className="px-2.5 py-0.5 rounded-full text-xs font-mono font-bold bg-rose-500/10 text-rose-400 border border-rose-500/20">
                {items.length} Quarantined
              </span>
            )}
          </div>
          <p className="text-xs text-zinc-400 mt-1">
            Deliveries that have exhausted their retry policy. Requeue individual items or bulk re-enqueue all failed deliveries.
          </p>
        </div>

        {items.length > 0 && (
          <button
            onClick={handleRetryAll}
            disabled={isRetryingAll}
            className="px-4 py-2.5 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs transition-all shadow-md flex items-center space-x-2 active:scale-95 disabled:opacity-50 self-start sm:self-auto"
          >
            {isRetryingAll ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <RefreshCw className="w-4 h-4" />
            )}
            <span>Retry All Quarantined ({items.length})</span>
          </button>
        )}
      </div>

      {/* Filter & Search Bar */}
      <div className="flex items-center space-x-3 bg-zinc-950 p-2 rounded-2xl border border-zinc-800">
        <div className="relative flex-1">
          <Search className="w-4 h-4 text-zinc-500 absolute left-3.5 top-3" />
          <input
            type="text"
            placeholder="Search dead-lettered items by ID, destination, or event type..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-transparent pl-10 pr-4 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none font-mono"
          />
        </div>
        <span className="text-xs font-mono text-zinc-500 px-3">
          {filteredItems.length} items
        </span>
      </div>

      {/* DLQ Table */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        {isLoading ? (
          <TableSkeleton rows={5} cols={6} />
        ) : filteredItems.length === 0 ? (
          <EmptyState
            icon={CheckCircle2}
            title="Dead Letter Queue is empty"
            description="All webhook deliveries have succeeded or are within active retry backoff windows."
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                  <th className="py-3 px-3">Delivery ID</th>
                  <th className="py-3 px-3">Event Type</th>
                  <th className="py-3 px-3">Target Destination</th>
                  <th className="py-3 px-3">Attempts Used</th>
                  <th className="py-3 px-3">Last Failure Reason</th>
                  <th className="py-3 px-3 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/60 font-mono">
                {filteredItems.map((item) => (
                  <tr
                    key={item.delivery_id}
                    onClick={() => navigate(`/deliveries/${item.delivery_id}`)}
                    className="hover:bg-zinc-900/40 cursor-pointer transition-colors group"
                  >
                    <td className="py-3.5 px-3 font-bold text-white group-hover:text-emerald-400 transition-colors">
                      {item.delivery_id.slice(0, 14)}...
                    </td>

                    <td className="py-3.5 px-3 text-purple-400 font-semibold">
                      {item.event_type}
                    </td>

                    <td className="py-3.5 px-3 font-sans font-semibold text-zinc-200">
                      {item.destination_name}
                    </td>

                    <td className="py-3.5 px-3 text-zinc-400">
                      {item.attempt_count} / {item.max_attempts}
                    </td>

                    <td className="py-3.5 px-3 text-rose-400 truncate max-w-xs">
                      {item.last_error || 'Exhausted retry budget (500/timeout)'}
                    </td>

                    <td className="py-3.5 px-3 text-right">
                      <div className="flex items-center justify-end space-x-2 font-sans">
                        <button
                          onClick={(e) => handleRequeue(e, item.delivery_id)}
                          className="px-2.5 py-1 rounded-lg bg-zinc-900 hover:bg-zinc-800 text-emerald-400 hover:text-emerald-300 text-xs font-semibold inline-flex items-center space-x-1 transition-colors"
                        >
                          <RotateCcw className="w-3 h-3" />
                          <span>Requeue</span>
                        </button>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            setDiscardTargetId(item.delivery_id);
                          }}
                          className="px-2.5 py-1 rounded-lg bg-rose-950/30 hover:bg-rose-900/40 text-rose-400 hover:text-rose-300 text-xs font-semibold inline-flex items-center space-x-1 transition-colors"
                        >
                          <Trash2 className="w-3 h-3" />
                          <span>Discard</span>
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Discard Confirmation Modal */}
      <ConfirmModal
        isOpen={!!discardTargetId}
        onClose={() => setDiscardTargetId(null)}
        onConfirm={handleConfirmDiscard}
        title="Discard Dead-Lettered Delivery?"
        description="This delivery will no longer be retried by the background worker. The underlying webhook event and historical attempt traces will remain preserved in the audit log."
        confirmText="Discard Delivery"
        variant="danger"
        isLoading={isDiscarding}
      />
    </div>
  );
};
