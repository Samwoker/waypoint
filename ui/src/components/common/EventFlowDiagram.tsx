import React from 'react';
import { ArrowRight, Radio, Send, Layers, CheckCircle2, XCircle, Clock, AlertTriangle, ShieldCheck } from 'lucide-react';

export interface FlowDeliveryBranch {
  destinationName: string;
  status: string;
  attemptCount: number;
  httpStatus?: number;
  latencyMs?: number;
}

interface EventFlowDiagramProps {
  sourceName: string;
  eventType: string;
  deliveries: FlowDeliveryBranch[];
  signatureValid?: boolean;
}

export const EventFlowDiagram: React.FC<EventFlowDiagramProps> = ({
  sourceName,
  eventType,
  deliveries,
  signatureValid = true,
}) => {
  return (
    <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
      <div className="flex items-center justify-between">
        <span className="text-[11px] font-mono font-bold uppercase tracking-wider text-zinc-400">
          Event Pipeline Fan-Out Trace
        </span>
        {signatureValid && (
          <span className="inline-flex items-center space-x-1 px-2 py-0.5 rounded-full text-[10px] font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            <ShieldCheck className="w-3 h-3" />
            <span>HMAC Signature Verified</span>
          </span>
        )}
      </div>

      <div className="flex flex-col lg:flex-row items-stretch lg:items-center gap-3 pt-2">
        {/* Step 1: Inbound Source */}
        <div className="flex-1 p-3.5 rounded-xl bg-zinc-950 border border-zinc-800 space-y-1">
          <div className="flex items-center space-x-2 text-zinc-400 text-[10px] font-mono uppercase">
            <Radio className="w-3.5 h-3.5 text-blue-400" />
            <span>1. Inbound Source</span>
          </div>
          <div className="text-xs font-bold text-white font-mono truncate">{sourceName}</div>
        </div>

        <div className="hidden lg:flex text-zinc-600">
          <ArrowRight className="w-4 h-4" />
        </div>

        {/* Step 2: Ingested Event */}
        <div className="flex-1 p-3.5 rounded-xl bg-zinc-950 border border-zinc-800 space-y-1">
          <div className="flex items-center space-x-2 text-zinc-400 text-[10px] font-mono uppercase">
            <Layers className="w-3.5 h-3.5 text-purple-400" />
            <span>2. Ingested Event</span>
          </div>
          <div className="text-xs font-bold text-white font-mono truncate">{eventType}</div>
        </div>

        <div className="hidden lg:flex text-zinc-600">
          <ArrowRight className="w-4 h-4" />
        </div>

        {/* Step 3: Fan-Out Deliveries to Destinations */}
        <div className="flex-[2] p-3.5 rounded-xl bg-zinc-950 border border-zinc-800 space-y-2">
          <div className="flex items-center justify-between text-zinc-400 text-[10px] font-mono uppercase">
            <div className="flex items-center space-x-2">
              <Send className="w-3.5 h-3.5 text-emerald-400" />
              <span>3. Subscriptions & Forwarded Deliveries</span>
            </div>
            <span className="font-bold text-zinc-300">({deliveries.length} endpoints)</span>
          </div>

          <div className="space-y-1.5 max-h-48 overflow-y-auto pr-1">
            {deliveries.length === 0 ? (
              <div className="text-[11px] text-zinc-500 italic py-1">
                No matching subscription routing rules for this event type.
              </div>
            ) : (
              deliveries.map((del, idx) => {
                const isDelivered = del.status === 'delivered';
                const isFailed = del.status === 'failed' || del.status === 'dead_letter';
                const isPending = del.status === 'pending' || del.status === 'processing';

                return (
                  <div
                    key={idx}
                    className="flex items-center justify-between p-2 rounded-lg bg-[#0e0e11] border border-zinc-800/80 text-xs"
                  >
                    <div className="flex items-center space-x-2 truncate">
                      {isDelivered && <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400 shrink-0" />}
                      {isFailed && <XCircle className="w-3.5 h-3.5 text-rose-400 shrink-0" />}
                      {isPending && <Clock className="w-3.5 h-3.5 text-amber-400 shrink-0" />}
                      <span className="font-semibold text-zinc-200 font-mono truncate">
                        {del.destinationName}
                      </span>
                    </div>

                    <div className="flex items-center space-x-2 text-[10px] font-mono shrink-0">
                      {del.httpStatus && (
                        <span
                          className={`px-1.5 py-0.5 rounded font-bold ${
                            del.httpStatus >= 200 && del.httpStatus < 300
                              ? 'bg-emerald-500/10 text-emerald-400'
                              : 'bg-rose-500/10 text-rose-400'
                          }`}
                        >
                          HTTP {del.httpStatus}
                        </span>
                      )}
                      {del.latencyMs !== undefined && (
                        <span className="text-zinc-500">{del.latencyMs}ms</span>
                      )}
                      <span
                        className={`px-2 py-0.5 rounded-full capitalize font-semibold ${
                          isDelivered
                            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                            : isFailed
                            ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                            : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                        }`}
                      >
                        {del.status.replace('_', ' ')}
                      </span>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
