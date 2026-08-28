import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Activity,
  Search,
  Filter,
  ArrowRight,
  RotateCcw,
  CheckCircle2,
  XCircle,
  Clock,
  ChevronLeft,
  ChevronRight,
  Loader2,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  fetchDeliveriesRequest,
  replayDeliveryRequest,
} from '../store/slices/deliveriesSlice';
import { api } from '../api/client';
import { Delivery } from '../types';
import { useToast } from '../context/ToastContext';
import { EmptyState, TableSkeleton } from '../components/common/Skeleton';

export const DeliveriesPage: React.FC = () => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const toast = useToast();
  const { deliveries, isLoading } = useAppSelector(
    (state) => state.deliveries
  );
  const [hasMore, setHasMore] = useState(false);
  const [nextCursor, setNextCursor] = useState<string | undefined>(undefined);

  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [cursorHistory, setCursorHistory] = useState<string[]>([]);
  const [currentCursorIndex, setCurrentCursorIndex] = useState<number>(0);

  useEffect(() => {
    dispatch(
      fetchDeliveriesRequest({
        status: statusFilter === 'all' ? undefined : statusFilter,
      })
    );
  }, [dispatch, statusFilter]);

  const handleNextPage = () => {
    if (nextCursor) {
      setCursorHistory((prev) => [...prev, nextCursor]);
      setCurrentCursorIndex((prev) => prev + 1);
      dispatch(
        fetchDeliveriesRequest({
          status: statusFilter === 'all' ? undefined : statusFilter,
        })
      );
    }
  };

  const handlePrevPage = () => {
    if (currentCursorIndex > 0) {
      const prevIdx = currentCursorIndex - 1;
      setCurrentCursorIndex(prevIdx);
      dispatch(
        fetchDeliveriesRequest({
          status: statusFilter === 'all' ? undefined : statusFilter,
        })
      );
    }
  };

  const handleReplay = async (e: React.MouseEvent, del: Delivery) => {
    e.stopPropagation();
    try {
      await api.replayDelivery(del.id);
      toast.success('Delivery Queued for Replay', 'Worker pipeline re-enqueued delivery task.');
      dispatch(
        fetchDeliveriesRequest({
          status: statusFilter === 'all' ? undefined : statusFilter,
        })
      );
    } catch (err: any) {
      toast.error('Replay failed', err.message);
    }
  };

  const filteredDeliveries = deliveries.filter(
    (d) =>
      d.id.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (d.destination_name || '').toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-extrabold text-white tracking-tight">
            Deliveries & Operational Traces
          </h1>
          <p className="text-xs text-zinc-400 mt-1">
            Real-time outbound dispatch attempts, latency monitoring, retry execution timelines, and replays.
          </p>
        </div>

        {/* Status Filter Buttons */}
        <div className="flex bg-zinc-950 p-1 rounded-xl border border-zinc-800 self-start sm:self-auto overflow-x-auto">
          {['all', 'delivered', 'failed', 'pending', 'dead_letter'].map((st) => (
            <button
              key={st}
              onClick={() => {
                setStatusFilter(st);
                setCurrentCursorIndex(0);
                setCursorHistory([]);
              }}
              className={`px-3 py-1.5 rounded-lg text-xs font-mono font-semibold capitalize transition-all shrink-0 ${
                statusFilter === st
                  ? 'bg-zinc-800 text-white shadow-sm border border-zinc-700/60'
                  : 'text-zinc-400 hover:text-zinc-200'
              }`}
            >
              {st.replace('_', ' ')}
            </button>
          ))}
        </div>
      </div>

      {/* Search & Filter Bar */}
      <div className="flex items-center space-x-3 bg-zinc-950 p-2 rounded-2xl border border-zinc-800">
        <div className="relative flex-1">
          <Search className="w-4 h-4 text-zinc-500 absolute left-3.5 top-3" />
          <input
            type="text"
            placeholder="Search deliveries by ID or destination name..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-transparent pl-10 pr-4 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none font-mono"
          />
        </div>
        <span className="text-xs font-mono text-zinc-500 px-3">
          {filteredDeliveries.length} deliveries
        </span>
      </div>

      {/* Deliveries Table */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        {isLoading ? (
          <TableSkeleton rows={6} cols={6} />
        ) : filteredDeliveries.length === 0 ? (
          <EmptyState
            icon={Activity}
            title="No deliveries found"
            description="No webhook deliveries match the current status filter."
          />
        ) : (
          <>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs border-collapse">
                <thead>
                  <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                    <th className="py-3 px-3">Delivery ID</th>
                    <th className="py-3 px-3">Target Destination</th>
                    <th className="py-3 px-3">Status</th>
                    <th className="py-3 px-3">Attempts Used</th>
                    <th className="py-3 px-3">Last Dispatched</th>
                    <th className="py-3 px-3 text-right">Quick Replay</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-800/60 font-mono">
                  {filteredDeliveries.map((del) => {
                    const isDelivered = del.status === 'delivered';
                    const isFailed = del.status === 'failed' || del.status === 'dead_letter';

                    return (
                      <tr
                        key={del.id}
                        onClick={() => navigate(`/deliveries/${del.id}`)}
                        className="hover:bg-zinc-900/40 cursor-pointer transition-colors group"
                      >
                        <td className="py-3.5 px-3 font-bold text-white group-hover:text-emerald-400 transition-colors">
                          {del.id.slice(0, 14)}...
                        </td>

                        <td className="py-3.5 px-3 font-sans font-semibold text-zinc-200">
                          {del.destination_name || del.destination_id.slice(0, 12)}
                        </td>

                        <td className="py-3.5 px-3">
                          <span
                            className={`px-2 py-0.5 rounded-full text-[10px] font-semibold capitalize ${
                              isDelivered
                                ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                                : isFailed
                                ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                                : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                            }`}
                          >
                            {del.status.replace('_', ' ')}
                          </span>
                        </td>

                        <td className="py-3.5 px-3 text-zinc-400">
                          {del.attempt_count} / {del.max_attempts}
                        </td>

                        <td className="py-3.5 px-3 text-zinc-400">
                          {new Date(del.created_at).toLocaleTimeString()}
                        </td>

                        <td className="py-3.5 px-3 text-right">
                          <button
                            onClick={(e) => handleReplay(e, del)}
                            className="px-2.5 py-1 rounded-lg bg-zinc-900 hover:bg-zinc-800 text-zinc-300 hover:text-white text-xs font-semibold inline-flex items-center space-x-1 transition-colors font-sans"
                          >
                            <RotateCcw className="w-3 h-3 text-zinc-400" />
                            <span>Replay</span>
                          </button>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>

            {/* Keyset Cursor Pagination */}
            <div className="flex items-center justify-between pt-4 border-t border-zinc-800/80 text-xs font-mono">
              <span className="text-zinc-500">
                Page {currentCursorIndex + 1}
              </span>

              <div className="flex items-center space-x-2">
                <button
                  type="button"
                  onClick={handlePrevPage}
                  disabled={currentCursorIndex === 0}
                  className="px-3 py-1.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-300 border border-zinc-800 disabled:opacity-40 flex items-center space-x-1"
                >
                  <ChevronLeft className="w-3.5 h-3.5" />
                  <span>Previous</span>
                </button>
                <button
                  type="button"
                  onClick={handleNextPage}
                  disabled={!hasMore}
                  className="px-3 py-1.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-300 border border-zinc-800 disabled:opacity-40 flex items-center space-x-1"
                >
                  <span>Next</span>
                  <ChevronRight className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
};
