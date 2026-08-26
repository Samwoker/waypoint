import React, { useEffect, useState } from 'react';
import {
  AlertCircle,
  Check,
  Copy,
  Key,
  Plus,
  Trash2,
  X,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  clearNewlyCreatedKey,
  createApiKeyRequest,
  fetchApiKeysRequest,
  fetchTenantUsageRequest,
  revokeApiKeyRequest,
} from '../store/slices/apiKeysSlice';

export const ApiKeysPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const { currentTenant } = useAppSelector((state) => state.auth);
  const { apiKeys, newlyCreatedKey, tenantUsage, isLoading } = useAppSelector((state) => state.apiKeys);
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [keyName, setKeyName] = useState('');
  const [expiresInDays, setExpiresInDays] = useState(30);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    dispatch(fetchApiKeysRequest());
    if (currentTenant) {
      dispatch(fetchTenantUsageRequest(currentTenant.id));
    }
  }, [dispatch, currentTenant]);

  const handleCreateKey = (e: React.FormEvent) => {
    e.preventDefault();
    dispatch(createApiKeyRequest({ name: keyName, expiresInDays }));
    setIsCreateOpen(false);
    setKeyName('');
  };

  const handleRevoke = (id: string) => {
    if (!confirm('Are you sure you want to revoke this API key? This action is immediate and cannot be undone.')) return;
    dispatch(revokeApiKeyRequest(id));
  };

  const handleCopyNewKey = () => {
    if (!newlyCreatedKey) return;
    navigator.clipboard.writeText(newlyCreatedKey.key);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-white tracking-tight">API Keys & Authentication</h1>
          <p className="text-xs text-zinc-400">Generate programmatic API keys (`X-Api-Key` or `Authorization: Bearer`) to interact with Waypoint.</p>
        </div>
        <button
          onClick={() => setIsCreateOpen(true)}
          className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-white text-zinc-950 font-semibold text-xs hover:bg-zinc-200 transition-colors shadow-md"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>Create New API Key</span>
        </button>
      </div>

      {/* Reveal New Key Banner */}
      {newlyCreatedKey && (
        <div className="p-5 rounded-2xl bg-amber-950/40 border border-amber-800/60 text-amber-200 text-xs space-y-3 animate-in fade-in">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2 text-amber-400 font-semibold">
              <AlertCircle className="w-4 h-4" />
              <span>Copy your new API Key now — it will not be shown again!</span>
            </div>
            <button
              onClick={() => dispatch(clearNewlyCreatedKey())}
              className="text-xs text-amber-400 hover:underline"
            >
              Dismiss
            </button>
          </div>
          <div className="p-3 bg-black/70 border border-amber-900/60 rounded-xl flex items-center justify-between font-mono text-xs">
            <span className="text-white select-all">{newlyCreatedKey.key}</span>
            <button
              onClick={handleCopyNewKey}
              className="flex items-center space-x-1 px-3 py-1 bg-amber-500 text-zinc-950 rounded font-semibold text-[11px] hover:bg-amber-400"
            >
              {copied ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
              <span>{copied ? 'Copied' : 'Copy Key'}</span>
            </button>
          </div>
        </div>
      )}

      {/* Tenant Consumption Quotas */}
      {tenantUsage && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-2">
            <span className="text-xs font-mono text-zinc-400">TENANT MONTHLY INGESTION</span>
            <div className="text-2xl font-bold font-mono text-white">
              {tenantUsage.total_events.toLocaleString()} <span className="text-xs font-normal text-zinc-500">/ 1,000,000 events</span>
            </div>
            <div className="w-full bg-zinc-900 rounded-full h-2 overflow-hidden border border-zinc-800">
              <div
                className="bg-emerald-400 h-2 rounded-full"
                style={{ width: `${Math.min(100, (tenantUsage.total_events / 1000000) * 100)}%` }}
              />
            </div>
          </div>

          <div className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 space-y-2">
            <span className="text-xs font-mono text-zinc-400">TOTAL DELIVERY ATTEMPTS</span>
            <div className="text-2xl font-bold font-mono text-white">
              {tenantUsage.total_delivery_attempts.toLocaleString()} <span className="text-xs font-normal text-zinc-500">attempts</span>
            </div>
            <div className="w-full bg-zinc-900 rounded-full h-2 overflow-hidden border border-zinc-800">
              <div
                className="bg-blue-400 h-2 rounded-full"
                style={{ width: `${Math.min(100, (tenantUsage.total_delivery_attempts / 1000000) * 100)}%` }}
              />
            </div>
          </div>
        </div>
      )}

      {/* API Keys Table */}
      <div className="rounded-2xl border border-zinc-800 bg-[#121215] overflow-hidden shadow-xl">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-zinc-900/80 border-b border-zinc-800 text-zinc-400">
              <tr>
                <th className="px-5 py-3 font-semibold">Key Name</th>
                <th className="px-5 py-3 font-semibold">Key Prefix</th>
                <th className="px-5 py-3 font-semibold">Status</th>
                <th className="px-5 py-3 font-semibold">Created</th>
                <th className="px-5 py-3 font-semibold">Expires</th>
                <th className="px-5 py-3 font-semibold text-right">Action</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 text-zinc-300">
              {apiKeys.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-5 py-10 text-center text-zinc-500 font-sans">
                    No active API keys created for this workspace yet.
                  </td>
                </tr>
              ) : (
                apiKeys.map((key) => (
                  <tr key={key.id} className="hover:bg-zinc-900/50 transition-colors">
                    <td className="px-5 py-3.5 text-white font-semibold flex items-center space-x-2">
                      <Key className="w-3.5 h-3.5 text-zinc-400" />
                      <span>{key.name}</span>
                    </td>
                    <td className="px-5 py-3.5 text-zinc-400 font-mono">
                      {key.key_prefix}...
                    </td>
                    <td className="px-5 py-3.5">
                      <span className="px-2 py-0.5 rounded-full text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">
                        ACTIVE
                      </span>
                    </td>
                    <td className="px-5 py-3.5 text-zinc-500">
                      {new Date(key.created_at).toLocaleDateString()}
                    </td>
                    <td className="px-5 py-3.5 text-zinc-500">
                      {key.expires_at ? new Date(key.expires_at).toLocaleDateString() : 'Never'}
                    </td>
                    <td className="px-5 py-3.5 text-right">
                      <button
                        onClick={() => handleRevoke(key.id)}
                        className="px-2.5 py-1 rounded bg-rose-950/40 hover:bg-rose-900/60 text-rose-400 text-[11px] font-sans transition-colors"
                      >
                        Revoke
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Create Key Modal */}
      {isCreateOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in">
          <div className="w-full max-w-md bg-[#121215] border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden">
            <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800 bg-zinc-900/40">
              <h3 className="text-sm font-semibold text-white">Generate API Key</h3>
              <button onClick={() => setIsCreateOpen(false)} className="text-zinc-400 hover:text-white">
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleCreateKey} className="p-6 space-y-4">
              <div>
                <label className="block text-xs font-mono text-zinc-400 mb-1">Key Description / Name</label>
                <input
                  type="text"
                  required
                  placeholder="CI/CD Ingestion Service"
                  value={keyName}
                  onChange={(e) => setKeyName(e.target.value)}
                  className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                />
              </div>

              <div>
                <label className="block text-xs font-mono text-zinc-400 mb-1">Expiration</label>
                <select
                  value={expiresInDays}
                  onChange={(e) => setExpiresInDays(Number(e.target.value))}
                  className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                >
                  <option value={30}>30 Days</option>
                  <option value={90}>90 Days</option>
                  <option value={365}>1 Year</option>
                  <option value={0}>Never Expire</option>
                </select>
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
                  Generate Key
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
