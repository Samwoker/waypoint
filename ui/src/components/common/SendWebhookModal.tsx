import React, { useState } from 'react';
import { Check, Loader2, Send, X } from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../../store/hooks';
import { clearSendResult, sendWebhookRequest } from '../../store/slices/eventsSlice';

interface SendWebhookModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
  initialSlug?: string;
}

export const SendWebhookModal: React.FC<SendWebhookModalProps> = ({
  isOpen,
  onClose,
  onSuccess,
  initialSlug = 'default-inbound',
}) => {
  const dispatch = useAppDispatch();
  const { isSending, sendResult, error } = useAppSelector((state) => state.events);
  const [slug, setSlug] = useState(initialSlug);
  const [eventType, setEventType] = useState('payment.completed');
  const [payloadText, setPayloadText] = useState(
    JSON.stringify(
      {
        id: `evt_${Date.now()}`,
        amount: 249.99,
        currency: 'USD',
        customer: {
          id: 'cus_8899',
          name: 'Sarah Jenkins',
          email: 'sarah@example.com',
        },
        status: 'succeeded',
      },
      null,
      2
    )
  );

  if (!isOpen) return null;

  const handleSend = () => {
    try {
      const parsed = JSON.parse(payloadText);
      dispatch(
        sendWebhookRequest({
          slug,
          payload: parsed,
          headers: { 'X-Event-Type': eventType },
        })
      );
      if (onSuccess) onSuccess();
    } catch (e: any) {
      alert(`Invalid JSON: ${e.message}`);
    }
  };

  const handleClose = () => {
    dispatch(clearSendResult());
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-150">
      <div className="w-full max-w-2xl bg-[#121215] border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800 bg-zinc-900/40">
          <div className="flex items-center space-x-2.5">
            <div className="p-2 rounded-lg bg-zinc-800 border border-zinc-700 text-white">
              <Send className="w-4 h-4" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-white">Dispatch Inbound Webhook</h3>
              <p className="text-xs text-zinc-400">Send a live payload directly to `/hooks/:slug`</p>
            </div>
          </div>
          <button
            onClick={handleClose}
            className="p-1 text-zinc-400 hover:text-white rounded-lg hover:bg-zinc-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div className="p-6 space-y-4 max-h-[75vh] overflow-y-auto">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-mono text-zinc-400 mb-1.5">Source Slug</label>
              <div className="flex rounded-lg border border-zinc-800 bg-[#09090b] overflow-hidden focus-within:border-zinc-600">
                <span className="px-2.5 py-2 text-xs font-mono text-zinc-600 bg-zinc-900/80 border-r border-zinc-800">
                  /hooks/
                </span>
                <input
                  type="text"
                  value={slug}
                  onChange={(e) => setSlug(e.target.value)}
                  placeholder="stripe-webhook"
                  className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-transparent focus:outline-none"
                />
              </div>
            </div>

            <div>
              <label className="block text-xs font-mono text-zinc-400 mb-1.5">Event Type Header</label>
              <input
                type="text"
                value={eventType}
                onChange={(e) => setEventType(e.target.value)}
                placeholder="order.created"
                className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
              />
            </div>
          </div>

          <div>
            <div className="flex items-center justify-between mb-1.5">
              <label className="text-xs font-mono text-zinc-400">JSON Payload</label>
              <span className="text-[10px] font-mono text-zinc-500">application/json</span>
            </div>
            <textarea
              rows={8}
              value={payloadText}
              onChange={(e) => setPayloadText(e.target.value)}
              className="w-full p-3 text-xs font-mono text-zinc-200 bg-[#09090b] border border-zinc-800 rounded-xl focus:outline-none focus:border-zinc-600 resize-none"
            />
          </div>

          {error && (
            <div className="p-3 text-xs rounded-lg bg-rose-950/40 border border-rose-800/60 text-rose-300">
              {error}
            </div>
          )}

          {sendResult && (
            <div className="p-3.5 text-xs rounded-xl bg-emerald-950/30 border border-emerald-800/50 text-emerald-200 space-y-1.5 animate-in fade-in">
              <div className="flex items-center space-x-2 font-semibold text-emerald-400">
                <Check className="w-4 h-4" />
                <span>Webhook Ingested (202 Accepted)</span>
              </div>
              <pre className="font-mono text-[11px] bg-black/40 p-2 rounded border border-emerald-900/40 overflow-x-auto">
                {JSON.stringify(sendResult, null, 2)}
              </pre>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end space-x-3 px-6 py-4 border-t border-zinc-800 bg-zinc-900/40">
          <button
            onClick={handleClose}
            className="px-4 py-2 text-xs font-medium text-zinc-400 hover:text-zinc-200 transition-colors"
          >
            Close
          </button>
          <button
            onClick={handleSend}
            disabled={isSending}
            className="flex items-center space-x-2 px-4 py-2 text-xs font-medium rounded-lg bg-white text-zinc-900 hover:bg-zinc-200 disabled:opacity-50 transition-all font-semibold"
          >
            {isSending ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Send className="w-3.5 h-3.5" />}
            <span>{isSending ? 'Ingesting...' : 'Send Webhook'}</span>
          </button>
        </div>
      </div>
    </div>
  );
};
