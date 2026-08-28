import React, { useEffect, useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import {
  Layers,
  ArrowLeft,
  RotateCcw,
  CheckCircle2,
  XCircle,
  Clock,
  Radio,
  Send,
  Lock,
  Copy,
  Check,
  ChevronDown,
  ChevronUp,
  Loader2,
  ShieldCheck,
  FileCode,
} from 'lucide-react';
import { api } from '../api/client';
import { EventDetail, EventDeliveryItem, RawEventPayload, Source } from '../types';
import { useToast } from '../context/ToastContext';
import { EventFlowDiagram } from '../components/common/EventFlowDiagram';
import { CodeBlock } from '../components/common/CodeBlock';
import { Skeleton } from '../components/common/Skeleton';

export const EventDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const toast = useToast();

  const [event, setEvent] = useState<EventDetail | null>(null);
  const [source, setSource] = useState<Source | null>(null);
  const [deliveries, setDeliveries] = useState<EventDeliveryItem[]>([]);
  const [rawPayload, setRawPayload] = useState<RawEventPayload | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const [isRawExpanded, setIsRawExpanded] = useState<boolean>(false);
  const [isLoadingRaw, setIsLoadingRaw] = useState<boolean>(false);
  const [isReplaying, setIsReplaying] = useState<boolean>(false);
  const [copiedPayload, setCopiedPayload] = useState<boolean>(false);

  const fetchEventData = async () => {
    if (!id) return;
    try {
      setIsLoading(true);
      const [evt, dels] = await Promise.all([
        api.getEvent(id),
        api.getEventDeliveries(id),
      ]);
      setEvent(evt);
      setDeliveries(dels);

      if (evt.source_id) {
        try {
          const src = await api.getSource(evt.source_id);
          setSource(src);
        } catch (_) {}
      }
    } catch (err: any) {
      toast.error('Failed to load event details', err.message);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchEventData();
  }, [id]);

  const handleToggleRaw = async () => {
    if (!isRawExpanded && !rawPayload && id) {
      try {
        setIsLoadingRaw(true);
        const raw = await api.getEventRaw(id);
        setRawPayload(raw);
      } catch (err: any) {
        toast.error('Failed to load raw payload', err.message);
      } finally {
        setIsLoadingRaw(false);
      }
    }
    setIsRawExpanded(!isRawExpanded);
  };

  const handleReplayEvent = async () => {
    if (!id) return;
    try {
      setIsReplaying(true);
      const res = await api.replayEvent(id);
      toast.success(
        'Event Replayed',
        `Re-enqueued ${res.deliveries_created} downstream delivery tasks.`
      );
      fetchEventData();
    } catch (err: any) {
      toast.error('Replay failed', err.message);
    } finally {
      setIsReplaying(false);
    }
  };

  const handleCopyPayload = () => {
    if (!rawPayload) return;
    navigator.clipboard.writeText(rawPayload.payload);
    setCopiedPayload(true);
    toast.success('Payload copied to clipboard');
    setTimeout(() => setCopiedPayload(false), 2000);
  };

  if (isLoading) {
    return (
      <div className="p-8 max-w-7xl mx-auto space-y-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-48 w-full rounded-2xl" />
        <Skeleton className="h-64 w-full rounded-2xl" />
      </div>
    );
  }

  if (!event) {
    return (
      <div className="p-8 max-w-7xl mx-auto text-center space-y-4">
        <p className="text-sm text-zinc-400">Webhook event not found.</p>
        <button
          onClick={() => navigate('/events')}
          className="px-4 py-2 text-xs font-semibold rounded-xl bg-zinc-800 text-white"
        >
          Back to Events Stream
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
            onClick={() => navigate('/events')}
            className="p-2 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-white border border-zinc-800 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div className="space-y-0.5">
            <div className="flex items-center space-x-2.5">
              <h1 className="text-xl font-bold text-white tracking-tight font-mono">
                {event.event_type}
              </h1>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                Ingested
              </span>
            </div>
            <p className="text-xs text-zinc-400 font-mono">ID: {event.id}</p>
          </div>
        </div>

        <div className="flex items-center space-x-2.5">
          <button
            onClick={handleReplayEvent}
            disabled={isReplaying}
            className="px-4 py-2 rounded-xl bg-zinc-100 text-zinc-950 hover:bg-white text-xs font-semibold flex items-center space-x-2 transition-all shadow-md active:scale-95 disabled:opacity-50"
          >
            {isReplaying ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <RotateCcw className="w-3.5 h-3.5" />
            )}
            <span>Replay Event</span>
          </button>
        </div>
      </div>

      {/* Visual Pipeline Fan-Out Diagram */}
      <EventFlowDiagram
        sourceName={source?.name || 'Inbound Source'}
        eventType={event.event_type}
        deliveries={deliveries.map((d) => ({
          destinationName: d.destination_name,
          status: d.status,
          attemptCount: d.attempt_count,
        }))}
      />

      {/* Fan-Out Deliveries Table */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-bold text-white tracking-tight">
            Downstream Forwarded Deliveries
          </h3>
          <span className="text-xs font-mono text-zinc-500">
            {deliveries.length} destinations matched
          </span>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                <th className="py-2.5 px-3">Delivery ID</th>
                <th className="py-2.5 px-3">Destination</th>
                <th className="py-2.5 px-3">Status</th>
                <th className="py-2.5 px-3">Attempts</th>
                <th className="py-2.5 px-3">Delivered / Next Retry</th>
                <th className="py-2.5 px-3 text-right">Action</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 font-mono">
              {deliveries.length === 0 ? (
                <tr>
                  <td colSpan={6} className="py-6 text-center text-zinc-500 italic">
                    No active subscriptions matched this event type.
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
                    <td className="py-2.5 px-3 text-white font-semibold">
                      {del.destination_name}
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
                    <td className="py-2.5 px-3 text-zinc-400">{del.attempt_count}</td>
                    <td className="py-2.5 px-3 text-zinc-400">
                      {del.delivered_at
                        ? new Date(del.delivered_at).toLocaleTimeString()
                        : del.next_attempt_at
                        ? `Retry at ${new Date(del.next_attempt_at).toLocaleTimeString()}`
                        : '—'}
                    </td>
                    <td className="py-2.5 px-3 text-right text-zinc-500 hover:text-white">
                      Inspect Trace &rarr;
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Sensitive Raw Payload Section (Section 11) */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2.5">
            <div className="p-2 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/20">
              <Lock className="w-4 h-4" />
            </div>
            <div>
              <h3 className="text-sm font-bold text-white tracking-tight">
                Sensitive Raw Webhook Payload
              </h3>
              <p className="text-xs text-zinc-400">
                Encrypted inbound request payload and headers captured at ingestion.
              </p>
            </div>
          </div>

          <button
            onClick={handleToggleRaw}
            className="px-3.5 py-1.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-xs font-semibold text-zinc-200 flex items-center space-x-1.5 transition-colors"
          >
            {isLoadingRaw ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : isRawExpanded ? (
              <>
                <ChevronUp className="w-3.5 h-3.5" />
                <span>Hide Payload</span>
              </>
            ) : (
              <>
                <ChevronDown className="w-3.5 h-3.5" />
                <span>Inspect Payload</span>
              </>
            )}
          </button>
        </div>

        {isRawExpanded && rawPayload && (
          <div className="space-y-4 pt-2 animate-in fade-in">
            {/* Headers snippet */}
            {rawPayload.headers && Object.keys(rawPayload.headers).length > 0 && (
              <div className="space-y-1.5">
                <span className="text-[10px] font-mono text-zinc-500 uppercase tracking-wider">
                  Received HTTP Headers
                </span>
                <div className="p-3 bg-zinc-950 rounded-xl border border-zinc-800 font-mono text-xs text-zinc-300 overflow-x-auto">
                  <pre>{JSON.stringify(rawPayload.headers, null, 2)}</pre>
                </div>
              </div>
            )}

            {/* Raw JSON Body */}
            <div className="space-y-1.5">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-mono text-zinc-500 uppercase tracking-wider">
                  JSON Body
                </span>
                <button
                  onClick={handleCopyPayload}
                  className="px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-[11px] font-mono text-zinc-300 flex items-center space-x-1 transition-colors"
                >
                  {copiedPayload ? (
                    <>
                      <Check className="w-3 h-3 text-emerald-400" />
                      <span className="text-emerald-400">Copied</span>
                    </>
                  ) : (
                    <>
                      <Copy className="w-3 h-3" />
                      <span>Copy JSON</span>
                    </>
                  )}
                </button>
              </div>

              <CodeBlock
                title="Inbound Webhook Payload"
                singleLang="json"
                singleCode={rawPayload.payload}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
