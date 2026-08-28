import React, { useEffect, useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import {
  Activity,
  ArrowLeft,
  RotateCcw,
  CheckCircle2,
  XCircle,
  Clock,
  Send,
  Layers,
  AlertTriangle,
  Loader2,
  ExternalLink,
  ChevronDown,
  ChevronUp,
} from 'lucide-react';
import { api } from '../api/client';
import { DeliveryDetail, DeliveryAttempt } from '../types';
import { useToast } from '../context/ToastContext';
import { Skeleton } from '../components/common/Skeleton';

export const DeliveryDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const toast = useToast();

  const [delivery, setDelivery] = useState<DeliveryDetail | null>(null);
  const [attempts, setAttempts] = useState<DeliveryAttempt[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [isReplaying, setIsReplaying] = useState<boolean>(false);

  const fetchDeliveryData = async () => {
    if (!id) return;
    try {
      setIsLoading(true);
      const [del, atts] = await Promise.all([
        api.getDelivery(id),
        api.getDeliveryAttempts(id),
      ]);
      setDelivery(del);
      setAttempts(atts.length > 0 ? atts : del.attempts || []);
    } catch (err: any) {
      toast.error('Failed to load delivery trace', err.message);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchDeliveryData();
  }, [id]);

  const handleReplay = async () => {
    if (!id) return;
    try {
      setIsReplaying(true);
      await api.replayDelivery(id);
      toast.success('Delivery Queued for Replay', 'Worker pipeline will attempt redelivery.');
      fetchDeliveryData();
    } catch (err: any) {
      toast.error('Replay failed', err.message);
    } finally {
      setIsReplaying(false);
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

  if (!delivery) {
    return (
      <div className="p-8 max-w-7xl mx-auto text-center space-y-4">
        <p className="text-sm text-zinc-400">Delivery trace not found.</p>
        <button
          onClick={() => navigate('/deliveries')}
          className="px-4 py-2 text-xs font-semibold rounded-xl bg-zinc-800 text-white"
        >
          Back to Deliveries
        </button>
      </div>
    );
  }

  const isDelivered = delivery.status === 'delivered';
  const isFailed = delivery.status === 'failed' || delivery.status === 'dead_letter';

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Top Bar Navigation */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-center space-x-3">
          <button
            onClick={() => navigate('/deliveries')}
            className="p-2 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-white border border-zinc-800 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div className="space-y-0.5">
            <div className="flex items-center space-x-2.5">
              <h1 className="text-xl font-bold text-white tracking-tight">Delivery Trace Debugger</h1>
              <span
                className={`px-2 py-0.5 rounded-full text-[10px] font-mono font-semibold uppercase ${
                  isDelivered
                    ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                    : isFailed
                    ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                    : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                }`}
              >
                {delivery.status.replace('_', ' ')}
              </span>
            </div>
            <p className="text-xs text-zinc-400 font-mono">ID: {delivery.id}</p>
          </div>
        </div>

        <div className="flex items-center space-x-2.5">
          <button
            onClick={handleReplay}
            disabled={isReplaying}
            className="px-4 py-2 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 text-xs font-semibold flex items-center space-x-2 transition-all shadow-md active:scale-95 disabled:opacity-50"
          >
            {isReplaying ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <RotateCcw className="w-3.5 h-3.5" />
            )}
            <span>Replay Delivery</span>
          </button>
        </div>
      </div>

      {/* Linked Resources Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Linked Event Card */}
        <div
          onClick={() => navigate(`/events/${delivery.event_id}`)}
          className="p-5 rounded-2xl bg-[#121215] hover:bg-zinc-900/60 border border-zinc-800 cursor-pointer transition-all space-y-1.5 group"
        >
          <div className="flex items-center space-x-2 text-[10px] font-mono uppercase text-zinc-500">
            <Layers className="w-3.5 h-3.5 text-purple-400" />
            <span>Originating Webhook Event</span>
          </div>
          <div className="text-xs font-bold text-white font-mono group-hover:text-emerald-400 transition-colors truncate">
            {delivery.event_id}
          </div>
          <div className="text-[11px] text-zinc-500 flex items-center space-x-1">
            <span>Inspect Event Details</span>
            <ExternalLink className="w-3 h-3" />
          </div>
        </div>

        {/* Linked Destination Card */}
        <div
          onClick={() => navigate(`/destinations/${delivery.destination_id}`)}
          className="p-5 rounded-2xl bg-[#121215] hover:bg-zinc-900/60 border border-zinc-800 cursor-pointer transition-all space-y-1.5 group"
        >
          <div className="flex items-center space-x-2 text-[10px] font-mono uppercase text-zinc-500">
            <Send className="w-3.5 h-3.5 text-emerald-400" />
            <span>Target Destination Endpoint</span>
          </div>
          <div className="text-xs font-bold text-white font-mono group-hover:text-emerald-400 transition-colors truncate">
            {delivery.destination_id}
          </div>
          <div className="text-[11px] text-zinc-500 flex items-center space-x-1">
            <span>View Destination Health</span>
            <ExternalLink className="w-3 h-3" />
          </div>
        </div>

        {/* Attempt Counter Card */}
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-1.5">
          <div className="flex items-center space-x-2 text-[10px] font-mono uppercase text-zinc-500">
            <Activity className="w-3.5 h-3.5 text-blue-400" />
            <span>Execution Summary</span>
          </div>
          <div className="text-sm font-bold text-white font-mono">
            {delivery.attempt_count} of {delivery.max_attempts} attempts used
          </div>
          <div className="text-[11px] text-zinc-500 font-mono">
            {delivery.next_retry_at
              ? `Next retry scheduled for ${new Date(delivery.next_retry_at).toLocaleTimeString()}`
              : isDelivered
              ? 'Delivery finalized successfully'
              : 'Retry budget exhausted (DLQ)'}
          </div>
        </div>
      </div>

      {/* Deep Attempt History Execution Timeline */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-6">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h3 className="text-sm font-bold text-white tracking-tight">
              Attempt Execution Trace Timeline
            </h3>
            <p className="text-xs text-zinc-400">
              Chronological log of HTTP request attempts, response headers, and latency.
            </p>
          </div>
          <span className="text-xs font-mono text-zinc-500">
            {attempts.length} attempts recorded
          </span>
        </div>

        <div className="space-y-4">
          {attempts.length === 0 ? (
            <div className="p-8 text-center text-zinc-500 italic text-xs">
              No delivery execution attempts logged yet.
            </div>
          ) : (
            attempts.map((att) => {
              const isSuccess = att.http_status && att.http_status >= 200 && att.http_status < 300;

              return (
                <div
                  key={att.id || att.attempt_number}
                  className="p-5 rounded-xl bg-zinc-950 border border-zinc-800 space-y-3"
                >
                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-zinc-800/80 pb-3">
                    <div className="flex items-center space-x-3">
                      <div
                        className={`w-7 h-7 rounded-lg flex items-center justify-center font-mono font-bold text-xs ${
                          isSuccess
                            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                            : 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                        }`}
                      >
                        #{att.attempt_number}
                      </div>
                      <div>
                        <div className="text-xs font-bold text-white flex items-center space-x-2">
                          <span>Attempt #{att.attempt_number}</span>
                          {att.http_status && (
                            <span
                              className={`px-1.5 py-0.2 rounded text-[10px] font-mono ${
                                isSuccess
                                  ? 'bg-emerald-500/10 text-emerald-400'
                                  : 'bg-rose-500/10 text-rose-400'
                              }`}
                            >
                              HTTP {att.http_status}
                            </span>
                          )}
                        </div>
                        <div className="text-[10px] font-mono text-zinc-500">
                          {new Date(att.created_at).toLocaleString()}
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center space-x-3 text-xs font-mono text-zinc-400">
                      {att.latency_ms !== undefined && (
                        <span>Duration: {att.latency_ms}ms</span>
                      )}
                    </div>
                  </div>

                  {/* Error Message if failed */}
                  {att.error_message && (
                    <div className="p-3 rounded-lg bg-rose-950/30 border border-rose-800/30 text-rose-300 text-xs font-mono">
                      Error: {att.error_message}
                    </div>
                  )}

                  {/* Response Snippet */}
                  {att.response_body_snippet && (
                    <div className="space-y-1">
                      <span className="text-[10px] font-mono text-zinc-500 uppercase tracking-wider">
                        Response Body Snippet
                      </span>
                      <pre className="p-3 rounded-lg bg-[#0e0e11] border border-zinc-800 font-mono text-xs text-zinc-300 overflow-x-auto">
                        {att.response_body_snippet}
                      </pre>
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
