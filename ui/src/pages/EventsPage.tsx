import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Layers,
  Search,
  Filter,
  ArrowRight,
  Send,
  Radio,
  Clock,
  ChevronLeft,
  ChevronRight,
  RotateCcw,
  CheckCircle2,
  XCircle,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchEventsRequest } from '../store/slices/eventsSlice';
import { api } from '../api/client';
import { EventItem, PaginatedEvents } from '../types';
import { useToast } from '../context/ToastContext';
import { EmptyState, TableSkeleton } from '../components/common/Skeleton';

interface EventsPageProps {
  onOpenSendModal?: () => void;
}

export const EventsPage: React.FC<EventsPageProps> = ({ onOpenSendModal }) => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const toast = useToast();
  const { events, isLoading } = useAppSelector((state) => state.events);
  const [hasMore, setHasMore] = useState(false);
  const [nextCursor, setNextCursor] = useState<string | undefined>(undefined);

  const [searchQuery, setSearchQuery] = useState('');
  const [cursorHistory, setCursorHistory] = useState<string[]>([]);
  const [currentCursorIndex, setCurrentCursorIndex] = useState<number>(0);

  useEffect(() => {
    dispatch(fetchEventsRequest());
  }, [dispatch]);

  const handleNextPage = () => {
    if (nextCursor) {
      setCursorHistory((prev) => [...prev, nextCursor]);
      setCurrentCursorIndex((prev) => prev + 1);
      dispatch(fetchEventsRequest());
    }
  };

  const handlePrevPage = () => {
    if (currentCursorIndex > 0) {
      const prevIdx = currentCursorIndex - 1;
      setCurrentCursorIndex(prevIdx);
      dispatch(fetchEventsRequest());
    }
  };

  const filteredEvents = events.filter(
    (e) =>
      e.event_type.toLowerCase().includes(searchQuery.toLowerCase()) ||
      e.id.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-extrabold text-white tracking-tight">
            Live Webhook Events Stream
          </h1>
          <p className="text-xs text-zinc-400 mt-1">
            Real-time feed of all inbound webhook payloads, cryptographic verifications, and delivery fan-out.
          </p>
        </div>
        {onOpenSendModal && (
          <button
            onClick={onOpenSendModal}
            className="px-4 py-2.5 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs transition-all shadow-md flex items-center space-x-2 active:scale-95 self-start sm:self-auto"
          >
            <Send className="w-4 h-4" />
            <span>Send Test Webhook</span>
          </button>
        )}
      </div>

      {/* Filter & Search Bar */}
      <div className="flex items-center space-x-3 bg-zinc-950 p-2 rounded-2xl border border-zinc-800">
        <div className="relative flex-1">
          <Search className="w-4 h-4 text-zinc-500 absolute left-3.5 top-3" />
          <input
            type="text"
            placeholder="Filter events by event_type or event ID..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-transparent pl-10 pr-4 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none font-mono"
          />
        </div>
        <span className="text-xs font-mono text-zinc-500 px-3">
          {filteredEvents.length} events
        </span>
      </div>

      {/* Events Table */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        {isLoading ? (
          <TableSkeleton rows={6} cols={5} />
        ) : filteredEvents.length === 0 ? (
          <EmptyState
            icon={Layers}
            title="No webhook events received yet"
            description="Send a test webhook to an active inbound source to see events streamed and forwarded in real time."
            actionText="Send Test Webhook"
            onAction={onOpenSendModal}
          />
        ) : (
          <>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs border-collapse">
                <thead>
                  <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                    <th className="py-3 px-3">Event Type</th>
                    <th className="py-3 px-3">Event ID</th>
                    <th className="py-3 px-3">Status</th>
                    <th className="py-3 px-3">Received At</th>
                    <th className="py-3 px-3 text-right">Action</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-800/60 font-mono">
                  {filteredEvents.map((evt) => (
                    <tr
                      key={evt.id}
                      onClick={() => navigate(`/events/${evt.id}`)}
                      className="hover:bg-zinc-900/40 cursor-pointer transition-colors group"
                    >
                      <td className="py-3.5 px-3">
                        <div className="font-bold text-white font-mono group-hover:text-emerald-400 transition-colors flex items-center space-x-2">
                          <Layers className="w-3.5 h-3.5 text-purple-400 shrink-0" />
                          <span>{evt.event_type}</span>
                        </div>
                      </td>

                      <td className="py-3.5 px-3 text-zinc-400 font-mono">
                        {evt.id.slice(0, 16)}...
                      </td>

                      <td className="py-3.5 px-3">
                        <span className="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase">
                          {evt.status || 'Ingested'}
                        </span>
                      </td>

                      <td className="py-3.5 px-3 text-zinc-400">
                        {new Date(evt.received_at || evt.created_at).toLocaleString()}
                      </td>

                      <td className="py-3.5 px-3 text-right">
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            navigate(`/events/${evt.id}`);
                          }}
                          className="px-2.5 py-1 rounded-lg bg-zinc-900 hover:bg-zinc-800 text-zinc-300 text-xs font-semibold inline-flex items-center space-x-1 transition-colors font-sans"
                        >
                          <span>Inspect Fan-Out</span>
                          <ArrowRight className="w-3 h-3" />
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Keyset Cursor Pagination Controls */}
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
