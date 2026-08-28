import React, { useEffect, useState } from 'react';
import {
  Key,
  Plus,
  Trash2,
  Copy,
  Check,
  Shield,
  Clock,
  Calendar,
  AlertTriangle,
  Lock,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchApiKeysRequest } from '../store/slices/apiKeysSlice';
import { api } from '../api/client';
import { ApiKey, ApiKeyCreated } from '../types';
import { useToast } from '../context/ToastContext';
import { SecretRevealModal } from '../components/common/SecretRevealModal';
import { ConfirmModal } from '../components/common/ConfirmModal';
import { EmptyState, TableSkeleton } from '../components/common/Skeleton';

export const ApiKeysPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const toast = useToast();
  const { apiKeys: keys, isLoading } = useAppSelector((state) => state.apiKeys);

  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [createdKeyData, setCreatedKeyData] = useState<ApiKeyCreated | null>(null);
  const [revokeTargetId, setRevokeTargetId] = useState<string | null>(null);
  const [isRevoking, setIsRevoking] = useState<boolean>(false);

  // Form State
  const [keyName, setKeyName] = useState('');
  const [keyScope, setKeyScope] = useState<'full' | 'read_only'>('full');
  const [expirationDays, setExpirationDays] = useState<string>('90');

  useEffect(() => {
    dispatch(fetchApiKeysRequest());
  }, [dispatch]);

  const handleCreateKey = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!keyName.trim()) {
      toast.error('Validation Error', 'Key name is required.');
      return;
    }

    try {
      const expiresAt =
        expirationDays === 'never'
          ? undefined
          : new Date(Date.now() + Number(expirationDays) * 86400000).toISOString();

      const nameWithScope =
        keyScope === 'read_only' && !keyName.toLowerCase().includes('read_only')
          ? `${keyName} (read_only)`
          : keyName;

      const created = await api.createApiKey(nameWithScope, expiresAt);
      setCreatedKeyData(created);
      setIsCreateModalOpen(false);
      toast.success('API Key Generated', 'Store your API key now.');
      dispatch(fetchApiKeysRequest());
      setKeyName('');
    } catch (err: any) {
      toast.error('Failed to create API key', err.message);
    }
  };

  const handleConfirmRevoke = async () => {
    if (!revokeTargetId) return;
    try {
      setIsRevoking(true);
      await api.revokeApiKey(revokeTargetId);
      toast.success('API Key Revoked', 'The key has been deactivated immediately.');
      setRevokeTargetId(null);
      dispatch(fetchApiKeysRequest());
    } catch (err: any) {
      toast.error('Failed to revoke API key', err.message);
    } finally {
      setIsRevoking(false);
    }
  };

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-extrabold text-white tracking-tight">
            Programmatic API Keys
          </h1>
          <p className="text-xs text-zinc-400 mt-1">
            Manage scoped bearer tokens for automated CLI, backend dispatch, and webhook ingestion.
          </p>
        </div>
        <button
          onClick={() => setIsCreateModalOpen(true)}
          className="px-4 py-2.5 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs transition-all shadow-md flex items-center space-x-2 active:scale-95 self-start sm:self-auto"
        >
          <Plus className="w-4 h-4" />
          <span>Generate API Key</span>
        </button>
      </div>

      {/* API Keys Table */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        {isLoading ? (
          <TableSkeleton rows={4} cols={5} />
        ) : keys.length === 0 ? (
          <EmptyState
            icon={Key}
            title="No API keys generated"
            description="Generate a scoped programmatic key to authenticate with the Waypoint REST API."
            actionText="Generate API Key"
            onAction={() => setIsCreateModalOpen(true)}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                  <th className="py-3 px-3">Key Name</th>
                  <th className="py-3 px-3">Token Prefix</th>
                  <th className="py-3 px-3">Scope</th>
                  <th className="py-3 px-3">Last Used</th>
                  <th className="py-3 px-3">Created</th>
                  <th className="py-3 px-3 text-right">Revoke</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/60 font-mono">
                {keys.map((key) => {
                  const isReadOnly = key.name.toLowerCase().includes('read_only');

                  return (
                    <tr key={key.id} className="hover:bg-zinc-900/40 transition-colors">
                      <td className="py-3.5 px-3 font-sans font-bold text-white">
                        {key.name}
                      </td>

                      <td className="py-3.5 px-3 text-emerald-400 font-semibold">
                        {key.key_prefix}••••••••••••
                      </td>

                      <td className="py-3.5 px-3">
                        <span
                          className={`px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase ${
                            isReadOnly
                              ? 'bg-blue-500/10 text-blue-400 border border-blue-500/20'
                              : 'bg-purple-500/10 text-purple-400 border border-purple-500/20'
                          }`}
                        >
                          {isReadOnly ? 'read_only' : 'full'}
                        </span>
                      </td>

                      <td className="py-3.5 px-3 text-zinc-400">
                        {key.last_used_at
                          ? new Date(key.last_used_at).toLocaleDateString()
                          : 'Never used'}
                      </td>

                      <td className="py-3.5 px-3 text-zinc-400">
                        {new Date(key.created_at).toLocaleDateString()}
                      </td>

                      <td className="py-3.5 px-3 text-right">
                        <button
                          onClick={() => setRevokeTargetId(key.id)}
                          className="px-2.5 py-1 rounded-lg bg-rose-950/30 hover:bg-rose-900/40 text-rose-400 hover:text-rose-300 text-xs font-semibold inline-flex items-center space-x-1 transition-colors font-sans"
                        >
                          <Trash2 className="w-3 h-3" />
                          <span>Revoke</span>
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Create Key Modal */}
      {isCreateModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-150">
          <div className="bg-[#121215] border border-zinc-800 rounded-3xl max-w-md w-full p-6 shadow-2xl space-y-5">
            <div className="flex items-center space-x-2.5">
              <div className="p-2 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/20">
                <Key className="w-5 h-5" />
              </div>
              <div>
                <h3 className="text-base font-bold text-white">Generate Programmatic API Key</h3>
                <p className="text-xs text-zinc-400">Create scoped credential for automation</p>
              </div>
            </div>

            <form onSubmit={handleCreateKey} className="space-y-4 text-xs font-mono">
              <div className="space-y-1">
                <label className="text-zinc-400">Key Name</label>
                <input
                  type="text"
                  required
                  placeholder="CI Deployment Key"
                  value={keyName}
                  onChange={(e) => setKeyName(e.target.value)}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                />
              </div>

              <div className="space-y-1">
                <label className="text-zinc-400">Access Scope</label>
                <select
                  value={keyScope}
                  onChange={(e) => setKeyScope(e.target.value as any)}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                >
                  <option value="full">Full Access (Read & Write)</option>
                  <option value="read_only">Read-Only (Telemetry & Traces)</option>
                </select>
              </div>

              <div className="space-y-1">
                <label className="text-zinc-400">Expiration</label>
                <select
                  value={expirationDays}
                  onChange={(e) => setExpirationDays(e.target.value)}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                >
                  <option value="30">30 Days</option>
                  <option value="90">90 Days</option>
                  <option value="365">1 Year</option>
                  <option value="never">Never Expire</option>
                </select>
              </div>

              <div className="flex items-center justify-end space-x-2 pt-3 border-t border-zinc-800/80">
                <button
                  type="button"
                  onClick={() => setIsCreateModalOpen(false)}
                  className="px-4 py-2 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-300 font-semibold text-xs transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-5 py-2 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs transition-all shadow-md active:scale-95"
                >
                  Generate Key
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Revoke Confirmation Modal */}
      <ConfirmModal
        isOpen={!!revokeTargetId}
        onClose={() => setRevokeTargetId(null)}
        onConfirm={handleConfirmRevoke}
        title="Revoke API Key?"
        description="This API key will immediately become invalid. Any automated systems or background workers using this key will receive 401 Unauthorized errors."
        confirmText="Revoke API Key"
        variant="danger"
        isLoading={isRevoking}
      />

      {/* One-Time Key Reveal Screen */}
      {createdKeyData && (
        <SecretRevealModal
          isOpen={!!createdKeyData}
          onClose={() => setCreatedKeyData(null)}
          title="API Key Generated Successfully"
          subtitle="Copy your key now. For your security, this key cannot be retrieved again."
          secret={createdKeyData.raw_key}
          warning="Never commit this key to public source repositories or share it with unauthorized personnel."
        />
      )}
    </div>
  );
};
