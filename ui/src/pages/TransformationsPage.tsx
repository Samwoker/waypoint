import React, { useState } from 'react';
import {
  Loader2,
  Play,
  Sparkles,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  clearEvaluatedOutput,
  testTransformationRequest,
} from '../store/slices/transformationsSlice';

export const TransformationsPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const { evaluatedOutput, isEvaluating, error } = useAppSelector((state) => state.transformations);

  const PRESETS = {
    stripe: {
      name: 'Stripe Payment',
      payload: {
        id: 'evt_3MjjkwLkdIwHu7ix0snNq8KG',
        type: 'payment_intent.succeeded',
        data: {
          object: {
            id: 'pi_3MjjkwLkdIwHu7ix0B',
            amount: 10999,
            currency: 'usd',
            customer: 'cus_994411',
            status: 'succeeded',
          },
        },
      },
      template: JSON.stringify(
        {
          event_type: 'PAYMENT_RECEIVED',
          payment_id: '$.data.object.id',
          amount_cents: '$.data.object.amount',
          currency: '$.data.object.currency',
          customer_ref: '$.data.object.customer',
        },
        null,
        2
      ),
    },
    github: {
      name: 'GitHub Push',
      payload: {
        ref: 'refs/heads/main',
        repository: {
          name: 'waypoint-relay',
          owner: { name: 'samwoker' },
        },
        head_commit: {
          id: 'b5a6c7d8e9',
          message: 'feat: add interactive transformation studio',
          author: { email: 'dev@example.com' },
        },
      },
      template: JSON.stringify(
        {
          branch: '$.ref',
          repo_name: '$.repository.name',
          commit_sha: '$.head_commit.id',
          commit_message: '$.head_commit.message',
          author_email: '$.head_commit.author.email',
        },
        null,
        2
      ),
    },
    shopify: {
      name: 'Shopify Order',
      payload: {
        id: 820982911946154500,
        email: 'jon@doe.ca',
        total_price: '598.94',
        currency: 'USD',
        financial_status: 'paid',
      },
      template: JSON.stringify(
        {
          order_id: '$.id',
          buyer: '$.email',
          total: '$.total_price',
          status: '$.financial_status',
        },
        null,
        2
      ),
    },
  };

  const [selectedPreset, setSelectedPreset] = useState<'stripe' | 'github' | 'shopify'>('stripe');
  const [inputPayload, setInputPayload] = useState(JSON.stringify(PRESETS.stripe.payload, null, 2));
  const [template, setTemplate] = useState(PRESETS.stripe.template);

  const handleSelectPreset = (key: 'stripe' | 'github' | 'shopify') => {
    setSelectedPreset(key);
    setInputPayload(JSON.stringify(PRESETS[key].payload, null, 2));
    setTemplate(PRESETS[key].template);
    dispatch(clearEvaluatedOutput());
  };

  const handleRunTest = () => {
    try {
      const parsed = JSON.parse(inputPayload);
      dispatch(testTransformationRequest({ template, payload: parsed }));
    } catch (e: any) {
      alert(`Invalid JSON: ${e.message}`);
    }
  };

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <div className="flex items-center space-x-2.5">
            <h1 className="text-xl font-bold text-white tracking-tight">Transformation Studio</h1>
            <span className="text-xs font-mono px-2 py-0.5 rounded-full bg-violet-500/10 text-violet-400 border border-violet-500/20 font-medium">
              JSONPath Sandbox
            </span>
          </div>
          <p className="text-xs text-zinc-400 mt-1">
            Reshape incoming webhook payloads on the fly using JSONPath expressions before downstream delivery.
          </p>
        </div>

        {/* Preset Selectors */}
        <div className="flex items-center space-x-3">
          <div className="flex items-center space-x-1 bg-zinc-950 p-1 rounded-lg border border-zinc-800 text-xs">
            <span className="px-2 text-zinc-500 font-mono text-[10px]">Presets:</span>
            {(['stripe', 'github', 'shopify'] as const).map((key) => (
              <button
                key={key}
                onClick={() => handleSelectPreset(key)}
                className={`px-3 py-1 font-mono rounded-md transition-colors capitalize ${
                  selectedPreset === key
                    ? 'bg-zinc-800 text-white font-semibold shadow-sm'
                    : 'text-zinc-400 hover:text-zinc-200'
                }`}
              >
                {key}
              </button>
            ))}
          </div>

          <button
            onClick={handleRunTest}
            disabled={isEvaluating}
            className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-white text-zinc-950 font-semibold text-xs hover:bg-zinc-200 transition-all shadow-md active:scale-95 disabled:opacity-50"
          >
            {isEvaluating ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Play className="w-3.5 h-3.5 fill-current" />}
            <span>Evaluate Transformation</span>
          </button>
        </div>
      </div>

      {error && (
        <div className="p-3.5 rounded-xl bg-rose-950/40 border border-rose-800/60 text-rose-300 text-xs font-mono">
          Error: {error}
        </div>
      )}

      {/* 3-Pane Editor Sandbox */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-5">
        {/* Pane 1: Inbound Webhook Payload */}
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-3 flex flex-col h-[520px]">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono font-semibold text-zinc-300 flex items-center space-x-1.5">
              <span className="w-2 h-2 rounded-full bg-emerald-400" />
              <span>1. Inbound JSON Payload</span>
            </span>
            <span className="text-[10px] font-mono text-zinc-500">Source Event</span>
          </div>

          <textarea
            value={inputPayload}
            onChange={(e) => setInputPayload(e.target.value)}
            className="flex-1 w-full p-3 bg-[#09090b] text-zinc-200 font-mono text-xs border border-zinc-800/80 rounded-xl focus:outline-none focus:border-zinc-600 resize-none leading-relaxed"
          />
        </div>

        {/* Pane 2: JSONPath Template Mapping */}
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-3 flex flex-col h-[520px]">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono font-semibold text-zinc-300 flex items-center space-x-1.5">
              <span className="w-2 h-2 rounded-full bg-violet-400" />
              <span>2. JSONPath Template</span>
            </span>
            <span className="text-[10px] font-mono text-zinc-500">Mapping Rules</span>
          </div>

          <textarea
            value={template}
            onChange={(e) => setTemplate(e.target.value)}
            className="flex-1 w-full p-3 bg-[#09090b] text-zinc-200 font-mono text-xs border border-zinc-800/80 rounded-xl focus:outline-none focus:border-zinc-600 resize-none leading-relaxed"
          />
        </div>

        {/* Pane 3: Evaluated Output Preview */}
        <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-3 flex flex-col h-[520px]">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono font-semibold text-zinc-300 flex items-center space-x-1.5">
              <span className="w-2 h-2 rounded-full bg-blue-400" />
              <span>3. Transformed Output</span>
            </span>
            <span className="text-[10px] font-mono text-zinc-500">Delivered Body</span>
          </div>

          <div className="flex-1 w-full p-3 bg-[#09090b] border border-zinc-800/80 rounded-xl overflow-auto text-xs font-mono text-zinc-200">
            {evaluatedOutput ? (
              <pre className="leading-relaxed">
                <code>{JSON.stringify(evaluatedOutput, null, 2)}</code>
              </pre>
            ) : (
              <div className="h-full flex flex-col items-center justify-center text-zinc-500 text-center space-y-2 p-4">
                <Sparkles className="w-6 h-6 text-zinc-600" />
                <p>Click "Evaluate Transformation" above to run the live dry-run test.</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
