import React, { useEffect, useState } from 'react';
import {
  Check,
  CheckCircle2,
  Copy,
  RefreshCw,
  Search,
  Send,
  ShieldCheck,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchEventsRequest, selectEvent } from '../store/slices/eventsSlice';

export const EventsPage: React.FC<{ onOpenSendModal: () => void }> = ({ onOpenSendModal }) => {
  const dispatch = useAppDispatch();
  const { events, selectedEvent, isLoading } = useAppSelector((state) => state.events);
  const [search, setSearch] = useState('');
  const [activeTab, setActiveTab] = useState<'payload' | 'headers'>('payload');
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    dispatch(fetchEventsRequest());
    const interval = setInterval(() => dispatch(fetchEventsRequest()), 5000);
    return () => clearInterval(interval);
  }, [dispatch]);

  const filteredEvents = events.filter(
    (e) =>
      e.event_type.toLowerCase().includes(search.toLowerCase()) ||
      e.id.toLowerCase().includes(search.toLowerCase())
  );

  const handleCopyPayload = () => {
    if (!selectedEvent) return;
    navigator.clipboard.writeText(JSON.stringify(selectedEvent.payload, null, 2));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="h-[calc(100vh-64px)] flex flex-col overflow-hidden animate-in fade-in duration-150">
      {/* Top action / search bar */}
      <div className="px-6 py-3 border-b border-zinc-800 bg-[#0c0c0e] flex items-center justify-between shrink-0">
        <div className="flex items-center space-x-3 w-80">
          <div className="relative w-full">
            <Search className="w-4 h-4 text-zinc-500 absolute left-3 top-2.5" />
            <input
              type="text"
              placeholder="Filter by event type or ID..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full pl-9 pr-3 py-1.5 text-xs bg-zinc-900 border border-zinc-800 rounded-lg text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-zinc-700"
            />
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={() => dispatch(fetchEventsRequest())}
            className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-xs text-zinc-300 hover:text-white hover:bg-zinc-800 transition-colors"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
            <span>Poll Stream</span>
          </button>
          <button
            onClick={onOpenSendModal}
            className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-white text-zinc-950 font-semibold text-xs hover:bg-zinc-200 transition-colors"
          >
            <Send className="w-3.5 h-3.5" />
            <span>Dispatch Test Webhook</span>
          </button>
        </div>
      </div>

      {/* Split-pane Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Pane: Events list stream */}
        <div className="w-96 border-r border-zinc-800 bg-[#09090b] overflow-y-auto shrink-0 divide-y divide-zinc-800/60">
          {filteredEvents.length === 0 ? (
            <div className="p-8 text-center text-xs text-zinc-500">
              No webhook events found matching filter.
            </div>
          ) : (
            filteredEvents.map((evt) => {
              const isSelected = selectedEvent?.id === evt.id;
              return (
                <div
                  key={evt.id}
                  onClick={() => dispatch(selectEvent(evt))}
                  className={`p-3.5 cursor-pointer transition-colors ${
                    isSelected
                      ? 'bg-zinc-800/80 border-l-2 border-l-emerald-400'
                      : 'hover:bg-zinc-900/60'
                  }`}
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-xs font-mono font-semibold text-zinc-100 truncate max-w-[180px]">
                      {evt.event_type}
                    </span>
                    <span className="text-[10px] font-mono text-zinc-500">
                      {new Date(evt.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                    </span>
                  </div>
                  <div className="flex items-center justify-between text-[11px] font-mono text-zinc-400">
                    <span className="truncate max-w-[160px]">{evt.id}</span>
                    <span className="flex items-center space-x-1 text-emerald-400 text-[10px]">
                      <CheckCircle2 className="w-3 h-3" />
                      <span>Ingested</span>
                    </span>
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Right Pane: Inspector */}
        <div className="flex-1 flex flex-col bg-[#0c0c0e] overflow-hidden">
          {selectedEvent ? (
            <>
              {/* Inspector Header */}
              <div className="p-6 border-b border-zinc-800 bg-[#121215]/50 flex items-start justify-between">
                <div>
                  <div className="flex items-center space-x-2.5">
                    <span className="text-base font-mono font-bold text-white">
                      {selectedEvent.event_type}
                    </span>
                    <span className="text-xs font-mono px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                      Signature Verified
                    </span>
                  </div>
                  <div className="text-xs font-mono text-zinc-500 mt-1 flex items-center space-x-3">
                    <span>ID: {selectedEvent.id}</span>
                    <span>•</span>
                    <span>Received: {new Date(selectedEvent.created_at).toLocaleString()}</span>
                  </div>
                </div>

                <div className="flex items-center space-x-2">
                  <button
                    onClick={handleCopyPayload}
                    className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-xs font-mono text-zinc-300 hover:text-white transition-colors"
                  >
                    {copied ? (
                      <>
                        <Check className="w-3.5 h-3.5 text-emerald-400" />
                        <span className="text-emerald-400">Copied</span>
                      </>
                    ) : (
                      <>
                        <Copy className="w-3.5 h-3.5" />
                        <span>Copy JSON</span>
                      </>
                    )}
                  </button>
                </div>
              </div>

              {/* Inspector Tabs */}
              <div className="flex items-center space-x-4 px-6 border-b border-zinc-800 bg-zinc-950/40 text-xs font-medium">
                <button
                  onClick={() => setActiveTab('payload')}
                  className={`py-2.5 border-b-2 font-mono transition-colors ${
                    activeTab === 'payload'
                      ? 'border-white text-white font-semibold'
                      : 'border-transparent text-zinc-500 hover:text-zinc-300'
                  }`}
                >
                  Raw Payload
                </button>
                <button
                  onClick={() => setActiveTab('headers')}
                  className={`py-2.5 border-b-2 font-mono transition-colors ${
                    activeTab === 'headers'
                      ? 'border-white text-white font-semibold'
                      : 'border-transparent text-zinc-500 hover:text-zinc-300'
                  }`}
                >
                  Inbound Headers
                </button>
              </div>

              {/* Tab Content */}
              <div className="flex-1 p-6 overflow-y-auto bg-[#09090b]">
                {activeTab === 'payload' ? (
                  <pre className="p-4 rounded-xl bg-[#0c0c0e] border border-zinc-800 text-xs font-mono text-zinc-200 overflow-x-auto leading-relaxed">
                    <code>{JSON.stringify(selectedEvent.payload, null, 2)}</code>
                  </pre>
                ) : (
                  <div className="space-y-3">
                    <div className="p-3 rounded-xl bg-emerald-950/20 border border-emerald-800/40 flex items-center space-x-2 text-xs text-emerald-400">
                      <ShieldCheck className="w-4 h-4" />
                      <span>HMAC-SHA256 signature was computed and validated successfully.</span>
                    </div>

                    <div className="rounded-xl border border-zinc-800 bg-[#0c0c0e] overflow-hidden">
                      <table className="w-full text-left text-xs font-mono">
                        <thead className="bg-zinc-900/60 border-b border-zinc-800 text-zinc-400">
                          <tr>
                            <th className="px-4 py-2.5">Header Name</th>
                            <th className="px-4 py-2.5">Value</th>
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-zinc-800/60 text-zinc-300">
                          {selectedEvent.headers && Object.entries(selectedEvent.headers).map(([k, v]) => (
                            <tr key={k}>
                              <td className="px-4 py-2 text-zinc-400 font-semibold">{k}</td>
                              <td className="px-4 py-2 text-zinc-200 truncate max-w-xs">{String(v)}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className="flex-1 flex items-center justify-center text-xs text-zinc-500">
              Select an event from the stream to inspect details.
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
