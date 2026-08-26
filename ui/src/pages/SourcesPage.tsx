import React, { useEffect, useState } from 'react';
import {
  Check,
  CheckCircle2,
  Copy,
  Plus,
  Radio,
  RotateCw,
  X,
  XCircle,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  clearSelectedSource,
  createSourceRequest,
  fetchSourcesRequest,
  fetchVerificationLogsRequest,
  rotateSecretRequest,
} from '../store/slices/sourcesSlice';
import { Source } from '../types';

export const SourcesPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const { sources, selectedSource, verificationLogs, generatedSecret, isLoading } = useAppSelector(
    (state) => state.sources
  );
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [copiedSlug, setCopiedSlug] = useState<string | null>(null);

  // Form states
  const [name, setName] = useState('');
  const [slug, setSlug] = useState('');
  const [provider, setProvider] = useState('stripe');
  const [verificationType, setVerificationType] = useState('hmac_sha256');
  const [secret, setSecret] = useState('');

  useEffect(() => {
    dispatch(fetchSourcesRequest());
  }, [dispatch]);

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    dispatch(
      createSourceRequest({
        name,
        slug,
        provider,
        verification_type: verificationType,
        secret: secret || undefined,
      })
    );
    setIsCreateOpen(false);
    setName('');
    setSlug('');
    setSecret('');
  };

  const handleCopyUrl = (s: Source) => {
    const url = `${window.location.origin}/hooks/${s.slug}`;
    navigator.clipboard.writeText(url);
    setCopiedSlug(s.slug);
    setTimeout(() => setCopiedSlug(null), 2000);
  };

  const handleRotateSecret = (s: Source) => {
    if (!confirm(`Rotate signing secret for source "${s.name}"? Existing webhooks will need updated keys.`)) return;
    dispatch(rotateSecretRequest(s.id));
  };

  const handleViewLogs = (s: Source) => {
    dispatch(fetchVerificationLogsRequest(s));
  };

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-white tracking-tight">Inbound Webhook Sources</h1>
          <p className="text-xs text-zinc-400">Manage incoming webhook endpoints, signature validation algorithms, and signing keys.</p>
        </div>
        <button
          onClick={() => setIsCreateOpen(true)}
          className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-white text-zinc-950 font-semibold text-xs hover:bg-zinc-200 transition-colors shadow-md"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>New Inbound Source</span>
        </button>
      </div>

      {generatedSecret && (
        <div className="p-4 rounded-xl bg-amber-950/40 border border-amber-800/60 text-amber-300 text-xs space-y-1 animate-in fade-in">
          <div className="font-semibold text-amber-400">New Signing Secret Generated (Copy Now):</div>
          <code className="font-mono bg-black/60 px-2 py-1 rounded block text-white select-all">
            {generatedSecret}
          </code>
        </div>
      )}

      {/* Sources Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
        {sources.map((s) => (
          <div
            key={s.id}
            className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 hover:border-zinc-700 transition-all space-y-4 shadow-lg flex flex-col justify-between"
          >
            <div>
              <div className="flex items-start justify-between">
                <div className="flex items-center space-x-2.5">
                  <div className="p-2 rounded-lg bg-zinc-900 border border-zinc-800 text-emerald-400">
                    <Radio className="w-4 h-4" />
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold text-white">{s.name}</h3>
                    <span className="text-[10px] font-mono text-zinc-500 uppercase">{s.provider}</span>
                  </div>
                </div>
                <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                  Active
                </span>
              </div>

              {/* Endpoint URL bar */}
              <div className="mt-4 p-2 rounded-lg bg-zinc-950 border border-zinc-800 flex items-center justify-between text-xs font-mono">
                <span className="text-zinc-400 truncate max-w-[200px]">/hooks/{s.slug}</span>
                <button
                  onClick={() => handleCopyUrl(s)}
                  className="text-zinc-500 hover:text-white p-1 rounded transition-colors"
                >
                  {copiedSlug === s.slug ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                </button>
              </div>

              <div className="mt-3 text-xs text-zinc-400 space-y-1 font-mono">
                <div className="flex justify-between">
                  <span className="text-zinc-500">Verification:</span>
                  <span className="text-zinc-300">{s.verification_type}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-500">Secret:</span>
                  <span className="text-zinc-300">{s.has_secret ? 'Configured •••••' : 'None'}</span>
                </div>
              </div>
            </div>

            <div className="pt-3 border-t border-zinc-800/80 flex items-center justify-between text-xs">
              <button
                onClick={() => handleViewLogs(s)}
                className="text-zinc-400 hover:text-white font-medium"
              >
                Verification Logs
              </button>
              <button
                onClick={() => handleRotateSecret(s)}
                className="text-xs text-zinc-500 hover:text-zinc-300 flex items-center space-x-1"
              >
                <RotateCw className="w-3 h-3" />
                <span>Rotate Secret</span>
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Create Source Modal */}
      {isCreateOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in">
          <div className="w-full max-w-lg bg-[#121215] border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden">
            <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800 bg-zinc-900/40">
              <h3 className="text-sm font-semibold text-white">Create Inbound Webhook Source</h3>
              <button onClick={() => setIsCreateOpen(false)} className="text-zinc-400 hover:text-white">
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleCreate} className="p-6 space-y-4">
              <div>
                <label className="block text-xs font-mono text-zinc-400 mb-1">Source Name</label>
                <input
                  type="text"
                  required
                  placeholder="Stripe Production Hooks"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                />
              </div>

              <div>
                <label className="block text-xs font-mono text-zinc-400 mb-1">Slug (URL Endpoint)</label>
                <input
                  type="text"
                  required
                  placeholder="stripe-prod-payments"
                  value={slug}
                  onChange={(e) => setSlug(e.target.value)}
                  className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-mono text-zinc-400 mb-1">Provider Type</label>
                  <select
                    value={provider}
                    onChange={(e) => setProvider(e.target.value)}
                    className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                  >
                    <option value="stripe">Stripe</option>
                    <option value="github">GitHub</option>
                    <option value="shopify">Shopify</option>
                    <option value="generic">Generic / Custom</option>
                  </select>
                </div>

                <div>
                  <label className="block text-xs font-mono text-zinc-400 mb-1">Verification Algorithm</label>
                  <select
                    value={verificationType}
                    onChange={(e) => setVerificationType(e.target.value)}
                    className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                  >
                    <option value="hmac_sha256">HMAC-SHA256</option>
                    <option value="stripe">Stripe v1 Signature</option>
                    <option value="github">GitHub X-Hub Signature</option>
                    <option value="shopify">Shopify Base64 HMAC</option>
                    <option value="none">None (Passthrough)</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="block text-xs font-mono text-zinc-400 mb-1">Signing Secret (Optional)</label>
                <input
                  type="text"
                  placeholder="whsec_xxxxxxxxxx"
                  value={secret}
                  onChange={(e) => setSecret(e.target.value)}
                  className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                />
              </div>

              <div className="flex items-center justify-end space-x-3 pt-3">
                <button
                  type="button"
                  onClick={() => setIsCreateOpen(false)}
                  className="px-4 py-2 text-xs text-zinc-400 hover:text-white"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 text-xs font-semibold rounded-lg bg-white text-zinc-950 hover:bg-zinc-200"
                >
                  Create Source
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Verification Logs Modal */}
      {selectedSource && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in">
          <div className="w-full max-w-2xl bg-[#121215] border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden">
            <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800 bg-zinc-900/40">
              <h3 className="text-sm font-semibold text-white">
                Verification Logs: {selectedSource.name}
              </h3>
              <button onClick={() => dispatch(clearSelectedSource())} className="text-zinc-400 hover:text-white">
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="p-6 max-h-96 overflow-y-auto space-y-2">
              {verificationLogs.length === 0 ? (
                <div className="p-8 text-center text-xs text-zinc-500 font-mono">
                  No verification history recorded for this source yet.
                </div>
              ) : (
                verificationLogs.map((log, idx) => (
                  <div
                    key={idx}
                    className="p-3 rounded-xl bg-zinc-950 border border-zinc-800 flex items-center justify-between text-xs font-mono"
                  >
                    <div className="flex items-center space-x-2.5">
                      {log.signature_valid ? (
                        <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                      ) : (
                        <XCircle className="w-4 h-4 text-rose-400" />
                      )}
                      <span className={log.signature_valid ? 'text-zinc-200' : 'text-rose-300 font-semibold'}>
                        {log.signature_valid ? 'Signature Passed' : 'Invalid Signature'}
                      </span>
                    </div>
                    <span className="text-zinc-500">{new Date(log.received_at).toLocaleString()}</span>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
