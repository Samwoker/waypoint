import React, { useEffect } from 'react';
import {
  CheckCircle2,
  Loader2,
  RefreshCw,
  RotateCw,
  Trash2,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  clearDlqMessage,
  discardDlqItemRequest,
  fetchDlqRequest,
  retryAllDlqRequest,
} from '../store/slices/dlqSlice';

export const DlqPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const { items, isLoading, isRetryingAll, successMessage } = useAppSelector((state) => state.dlq);

  useEffect(() => {
    dispatch(fetchDlqRequest());
  }, [dispatch]);

  const handleRetryAll = () => {
    dispatch(retryAllDlqRequest());
  };

  const handleDiscard = (id: string) => {
    if (!confirm('Are you sure you want to discard this dead-lettered item?')) return;
    dispatch(discardDlqItemRequest(id));
  };

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6 animate-in fade-in duration-150">
      {/* Header & Bulk Actions */}
      <div className="flex items-center justify-between">
        <div>
          <div className="flex items-center space-x-2.5">
            <h1 className="text-xl font-bold text-white tracking-tight">Dead Letter Queue (DLQ)</h1>
            <span className="text-xs font-mono px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/20 font-medium">
              Quarantine Studio
            </span>
          </div>
          <p className="text-xs text-zinc-400 mt-1">
            Exhausted deliveries that failed all retry attempts. Replay individually or in bulk.
          </p>
        </div>

        <div className="flex items-center space-x-3">
          <button
            onClick={() => dispatch(fetchDlqRequest())}
            className="p-2 rounded-lg bg-zinc-900 border border-zinc-800 text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
          </button>

          <button
            onClick={handleRetryAll}
            disabled={isRetryingAll || items.length === 0}
            className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-semibold text-xs disabled:opacity-50 transition-all active:scale-95 shadow-md"
          >
            {isRetryingAll ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <RotateCw className="w-3.5 h-3.5" />}
            <span>Retry All Failed Deliveries</span>
          </button>
        </div>
      </div>

      {successMessage && (
        <div className="p-4 rounded-xl bg-emerald-950/40 border border-emerald-800/60 text-emerald-300 text-xs flex items-center justify-between animate-in fade-in">
          <div className="flex items-center space-x-2">
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            <span>{successMessage}</span>
          </div>
          <button onClick={() => dispatch(clearDlqMessage())} className="text-xs text-emerald-400 hover:underline">
            Dismiss
          </button>
        </div>
      )}

      {/* DLQ Records Table */}
      <div className="rounded-2xl border border-zinc-800 bg-[#121215] overflow-hidden shadow-xl">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-zinc-900/80 border-b border-zinc-800 text-zinc-400">
              <tr>
                <th className="px-5 py-3 font-semibold">Delivery ID</th>
                <th className="px-5 py-3 font-semibold">Event Type</th>
                <th className="px-5 py-3 font-semibold">Destination</th>
                <th className="px-5 py-3 font-semibold">Attempts</th>
                <th className="px-5 py-3 font-semibold">Last Error</th>
                <th className="px-5 py-3 font-semibold text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 text-zinc-300">
              {items.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-5 py-12 text-center text-zinc-500 font-sans">
                    <div className="flex flex-col items-center justify-center space-y-2">
                      <div className="p-3 rounded-full bg-zinc-900 border border-zinc-800 text-emerald-400">
                        <CheckCircle2 className="w-6 h-6" />
                      </div>
                      <span className="text-sm font-medium text-zinc-300">Dead Letter Queue is Empty</span>
                      <span className="text-xs text-zinc-500">All webhook deliveries are healthy and processing normally.</span>
                    </div>
                  </td>
                </tr>
              ) : (
                items.map((item) => (
                  <tr key={item.delivery_id} className="hover:bg-zinc-900/50 transition-colors">
                    <td className="px-5 py-3.5 text-zinc-200 font-medium truncate max-w-[140px]">
                      {item.delivery_id}
                    </td>
                    <td className="px-5 py-3.5 text-white font-semibold">
                      {item.event_type}
                    </td>
                    <td className="px-5 py-3.5 text-zinc-400 truncate max-w-[180px]">
                      {item.destination_name} ({item.destination_url})
                    </td>
                    <td className="px-5 py-3.5 text-rose-400">
                      {item.attempt_count} / {item.max_attempts}
                    </td>
                    <td className="px-5 py-3.5 text-rose-300 truncate max-w-[200px]">
                      {item.last_error || 'Exhausted retry budget'}
                    </td>
                    <td className="px-5 py-3.5 text-right space-x-2">
                      <button
                        onClick={() => handleDiscard(item.delivery_id)}
                        className="p-1 rounded bg-rose-950/40 hover:bg-rose-900/60 text-rose-400 transition-colors"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
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
