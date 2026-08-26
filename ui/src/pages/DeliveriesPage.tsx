import React, { useEffect } from 'react';
import {
  CheckCircle2,
  RefreshCw,
  RotateCw,
  X,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  clearSelectedDelivery,
  fetchDeliveriesRequest,
  replayDeliveryRequest,
  selectDeliveryRequest,
  setStatusFilter,
} from '../store/slices/deliveriesSlice';

export const DeliveriesPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const {
    deliveries,
    selectedDelivery,
    attempts,
    statusFilter,
    isLoading,
    isReplaying,
    replaySuccess,
  } = useAppSelector((state) => state.deliveries);

  useEffect(() => {
    dispatch(fetchDeliveriesRequest());
  }, [dispatch, statusFilter]);

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6 animate-in fade-in duration-150">
      {/* Top Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-white tracking-tight">Delivery Attempts & Trace Logs</h1>
          <p className="text-xs text-zinc-400">Inspect fan-out delivery statuses, HTTP latencies, and replay failed attempts.</p>
        </div>

        <div className="flex items-center space-x-2">
          {/* Status Filter */}
          <div className="flex items-center space-x-1 bg-zinc-950 p-1 rounded-lg border border-zinc-800 text-xs">
            {['all', 'delivered', 'pending', 'failed', 'dead_letter'].map((s) => (
              <button
                key={s}
                onClick={() => dispatch(setStatusFilter(s))}
                className={`px-3 py-1 font-mono rounded-md transition-colors capitalize ${
                  statusFilter === s
                    ? 'bg-zinc-800 text-white font-semibold shadow-sm'
                    : 'text-zinc-400 hover:text-zinc-200'
                }`}
              >
                {s.replace('_', ' ')}
              </button>
            ))}
          </div>

          <button
            onClick={() => dispatch(fetchDeliveriesRequest())}
            className="p-2 rounded-lg bg-zinc-900 border border-zinc-800 text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* Deliveries Table */}
      <div className="rounded-2xl border border-zinc-800 bg-[#121215] overflow-hidden shadow-xl">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-zinc-900/80 border-b border-zinc-800 text-zinc-400">
              <tr>
                <th className="px-5 py-3 font-semibold">Delivery ID</th>
                <th className="px-5 py-3 font-semibold">Event Type</th>
                <th className="px-5 py-3 font-semibold">Status</th>
                <th className="px-5 py-3 font-semibold">Attempts</th>
                <th className="px-5 py-3 font-semibold">Timestamp</th>
                <th className="px-5 py-3 font-semibold text-right">Action</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 text-zinc-300">
              {deliveries.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-5 py-10 text-center text-zinc-500 font-sans">
                    No deliveries found matching current filter.
                  </td>
                </tr>
              ) : (
                deliveries.map((del) => (
                  <tr
                    key={del.id}
                    onClick={() => dispatch(selectDeliveryRequest(del))}
                    className={`cursor-pointer transition-colors ${
                      selectedDelivery?.id === del.id ? 'bg-zinc-800/60' : 'hover:bg-zinc-900/50'
                    }`}
                  >
                    <td className="px-5 py-3.5 text-zinc-200 font-medium truncate max-w-[140px]">
                      {del.id}
                    </td>
                    <td className="px-5 py-3.5 text-white font-semibold">
                      {del.event_type || 'webhook.event'}
                    </td>
                    <td className="px-5 py-3.5">
                      <span
                        className={`px-2 py-0.5 rounded-full text-[10px] font-medium border ${
                          del.status === 'delivered'
                            ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                            : del.status === 'pending'
                            ? 'bg-amber-500/10 text-amber-400 border-amber-500/20'
                            : 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                        }`}
                      >
                        {del.status.toUpperCase()}
                      </span>
                    </td>
                    <td className="px-5 py-3.5 text-zinc-400">
                      {del.attempt_count} / {del.max_attempts}
                    </td>
                    <td className="px-5 py-3.5 text-zinc-500">
                      {new Date(del.created_at).toLocaleString()}
                    </td>
                    <td className="px-5 py-3.5 text-right">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          dispatch(replayDeliveryRequest(del.id));
                        }}
                        className="px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white transition-colors text-[11px] font-sans"
                      >
                        Replay
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Detail Attempt Trace Drawer */}
      {selectedDelivery && (
        <div className="p-6 rounded-2xl border border-zinc-800 bg-[#121215] space-y-5 animate-in slide-in-from-bottom-2 duration-150">
          <div className="flex items-center justify-between border-b border-zinc-800 pb-4">
            <div>
              <div className="flex items-center space-x-2.5">
                <span className="text-sm font-bold text-white font-mono">Trace: {selectedDelivery.id}</span>
                <span className="text-xs font-mono px-2 py-0.5 rounded bg-zinc-800 text-zinc-300">
                  Event: {selectedDelivery.event_id}
                </span>
              </div>
              <p className="text-xs text-zinc-400 mt-1 font-mono">
                Destination: {selectedDelivery.destination_id}
              </p>
            </div>

            <div className="flex items-center space-x-3">
              {replaySuccess && (
                <span className="text-xs text-emerald-400 font-medium flex items-center space-x-1">
                  <CheckCircle2 className="w-3.5 h-3.5" />
                  <span>Replay Dispatched!</span>
                </span>
              )}
              <button
                onClick={() => dispatch(replayDeliveryRequest(selectedDelivery.id))}
                disabled={isReplaying}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-white text-zinc-950 font-semibold text-xs hover:bg-zinc-200 transition-colors"
              >
                <RotateCw className={`w-3.5 h-3.5 ${isReplaying ? 'animate-spin' : ''}`} />
                <span>{isReplaying ? 'Replaying...' : 'Replay Delivery'}</span>
              </button>
              <button
                onClick={() => dispatch(clearSelectedDelivery())}
                className="p-1 text-zinc-500 hover:text-zinc-300 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          </div>

          {/* Attempt List */}
          <div className="space-y-3">
            <h3 className="text-xs font-mono font-semibold text-zinc-400 uppercase">
              Attempt Execution History ({attempts.length})
            </h3>

            {attempts.length === 0 ? (
              <div className="p-6 text-center text-xs text-zinc-500 bg-zinc-950 rounded-xl border border-zinc-800">
                No attempt records yet or delivery is pending in worker queue.
              </div>
            ) : (
              attempts.map((att) => (
                <div
                  key={att.id}
                  className="p-4 rounded-xl bg-zinc-950/80 border border-zinc-800 space-y-3"
                >
                  <div className="flex items-center justify-between text-xs font-mono">
                    <div className="flex items-center space-x-2">
                      <span className="font-semibold text-white">Attempt #{att.attempt_number}</span>
                      <span
                        className={`px-2 py-0.5 rounded text-[10px] ${
                          att.status === 'success'
                            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                            : 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                        }`}
                      >
                        HTTP {att.response_status || 'ERR'}
                      </span>
                    </div>
                    <div className="text-zinc-500 flex items-center space-x-3">
                      <span>{att.duration_ms ? `${att.duration_ms} ms` : ''}</span>
                      <span>{new Date(att.created_at).toLocaleString()}</span>
                    </div>
                  </div>

                  {att.error_message && (
                    <div className="p-2.5 rounded-lg bg-rose-950/30 border border-rose-900/40 text-xs text-rose-300 font-mono">
                      Error: {att.error_message}
                    </div>
                  )}

                  {att.response_body && (
                    <div>
                      <div className="text-[11px] font-mono text-zinc-500 mb-1">Response Body:</div>
                      <pre className="p-2.5 rounded-lg bg-[#09090b] border border-zinc-800 text-[11px] font-mono text-zinc-300 overflow-x-auto">
                        <code>{att.response_body}</code>
                      </pre>
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
};
