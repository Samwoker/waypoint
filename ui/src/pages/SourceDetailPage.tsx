import React, { useEffect, useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import {
  Radio,
  Copy,
  Check,
  RefreshCw,
  Trash2,
  Edit2,
  ShieldCheck,
  ShieldAlert,
  ArrowLeft,
  Zap,
  Layers,
  Clock,
  CheckCircle2,
  XCircle,
  ExternalLink,
  Plus,
} from 'lucide-react';
import { api } from '../api/client';
import { Source, Subscription, VerificationLog, EventItem } from '../types';
import { useToast } from '../context/ToastContext';
import { SecretRevealModal } from '../components/common/SecretRevealModal';
import { ConfirmModal } from '../components/common/ConfirmModal';
import { Skeleton, TableSkeleton } from '../components/common/Skeleton';

export const SourceDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const toast = useToast();

  const [source, setSource] = useState<Source | null>(null);
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [logs, setLogs] = useState<VerificationLog[]>([]);
  const [events, setEvents] = useState<EventItem[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const [copiedUrl, setCopiedUrl] = useState(false);
  const [newSecret, setNewSecret] = useState<string | null>(null);
  const [isRotating, setIsRotating] = useState(false);
  const [isRotateConfirmOpen, setIsRotateConfirmOpen] = useState(false);
  const [isDeleteConfirmOpen, setIsDeleteConfirmOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const fetchSourceDetails = async () => {
    if (!id) return;
    try {
      setIsLoading(true);
      const [src, allSubs, verifLogs, evtsRes] = await Promise.all([
        api.getSource(id),
        api.listSubscriptions(),
        api.getSourceVerificationLog(id, 15),
        api.listEvents(15),
      ]);
      setSource(src);
      setSubscriptions(allSubs.filter((s) => s.source_id === id));
      setLogs(verifLogs);
      setEvents(evtsRes.events.filter((e) => e.source_id === id));
    } catch (err: any) {
      toast.error('Failed to load source details', err.message);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchSourceDetails();
  }, [id]);

  const handleCopyUrl = () => {
    if (!source) return;
    const url = `${window.location.origin}/hooks/${source.slug}`;
    navigator.clipboard.writeText(url);
    setCopiedUrl(true);
    toast.success('Inbound URL copied to clipboard');
    setTimeout(() => setCopiedUrl(false), 2000);
  };

  const handleRotateSecret = async () => {
    if (!id) return;
    try {
      setIsRotating(true);
      const res = await api.rotateSourceSecret(id);
      setNewSecret(res.secret);
      setIsRotateConfirmOpen(false);
      toast.warning('Signing Secret Rotated', 'Existing integrations must be updated with the new secret.');
      fetchSourceDetails();
    } catch (err: any) {
      toast.error('Failed to rotate secret', err.message);
    } finally {
      setIsRotating(false);
    }
  };

  const handleDeleteSource = async () => {
    if (!id) return;
    try {
      setIsDeleting(true);
      await api.deleteSource(id);
      setIsDeleteConfirmOpen(false);
      toast.success('Source deleted successfully');
      navigate('/sources');
    } catch (err: any) {
      toast.error('Failed to delete source', err.message);
    } finally {
      setIsDeleting(false);
    }
  };

  if (isLoading) {
    return (
      <div className="p-8 max-w-7xl mx-auto space-y-6 animate-in fade-in">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-40 w-full rounded-2xl" />
        <TableSkeleton rows={4} />
      </div>
    );
  }

  if (!source) {
    return (
      <div className="p-8 max-w-7xl mx-auto text-center space-y-4">
        <p className="text-sm text-zinc-400">Source not found.</p>
        <button
          onClick={() => navigate('/sources')}
          className="px-4 py-2 text-xs font-semibold rounded-xl bg-zinc-800 text-white"
        >
          Back to Sources
        </button>
      </div>
    );
  }

  const webhookUrl = `${window.location.origin}/hooks/${source.slug}`;

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Back Button & Top Action Bar */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-center space-x-3">
          <button
            onClick={() => navigate('/sources')}
            className="p-2 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-white border border-zinc-800 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div className="space-y-0.5">
            <div className="flex items-center space-x-2.5">
              <h1 className="text-xl font-bold text-white tracking-tight">{source.name}</h1>
              <span
                className={`px-2 py-0.5 rounded-full text-[10px] font-mono font-semibold uppercase ${
                  source.is_active
                    ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                    : 'bg-zinc-800 text-zinc-400 border border-zinc-700'
                }`}
              >
                {source.is_active ? 'Active' : 'Inactive'}
              </span>
            </div>
            <p className="text-xs text-zinc-400">{source.description || 'No description provided.'}</p>
          </div>
        </div>

        <div className="flex items-center space-x-2.5">
          <button
            onClick={() => setIsRotateConfirmOpen(true)}
            className="px-3 py-2 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-200 hover:text-white text-xs font-semibold flex items-center space-x-1.5 transition-colors"
          >
            <RefreshCw className="w-3.5 h-3.5" />
            <span>Rotate Secret</span>
          </button>
          <button
            onClick={() => setIsDeleteConfirmOpen(true)}
            className="px-3 py-2 rounded-xl bg-rose-950/40 hover:bg-rose-900/50 border border-rose-800/40 text-rose-300 text-xs font-semibold flex items-center space-x-1.5 transition-colors"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>Delete Source</span>
          </button>
        </div>
      </div>

      {/* Overview & Ingestion Endpoint Card */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
          <span className="text-[11px] font-mono font-bold text-zinc-400 uppercase tracking-wider">
            Public Inbound Webhook Endpoint
          </span>

          <div className="flex items-center space-x-2 bg-zinc-950 p-2.5 rounded-xl border border-zinc-800">
            <code className="flex-1 font-mono text-xs text-emerald-400 truncate select-all px-2">
              {webhookUrl}
            </code>
            <button
              onClick={handleCopyUrl}
              className="px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-white text-xs font-semibold flex items-center space-x-1.5 transition-colors shrink-0"
            >
              {copiedUrl ? (
                <>
                  <Check className="w-3.5 h-3.5 text-emerald-400" />
                  <span className="text-emerald-400">Copied</span>
                </>
              ) : (
                <>
                  <Copy className="w-3.5 h-3.5" />
                  <span>Copy URL</span>
                </>
              )}
            </button>
          </div>

          <div className="grid grid-cols-2 md:grid-cols-4 gap-3 pt-2 text-xs">
            <div className="p-3 rounded-xl bg-zinc-950 border border-zinc-800 space-y-1">
              <span className="text-[10px] font-mono text-zinc-500 uppercase">Provider</span>
              <div className="font-semibold text-white capitalize">{source.provider}</div>
            </div>
            <div className="p-3 rounded-xl bg-zinc-950 border border-zinc-800 space-y-1">
              <span className="text-[10px] font-mono text-zinc-500 uppercase">Verification Type</span>
              <div className="font-semibold text-white capitalize">{source.verification_type}</div>
            </div>
            <div className="p-3 rounded-xl bg-zinc-950 border border-zinc-800 space-y-1">
              <span className="text-[10px] font-mono text-zinc-500 uppercase">Tolerance</span>
              <div className="font-semibold text-white font-mono">
                {source.timestamp_tolerance_secs ? `${source.timestamp_tolerance_secs}s` : '300s'}
              </div>
            </div>
            <div className="p-3 rounded-xl bg-zinc-950 border border-zinc-800 space-y-1">
              <span className="text-[10px] font-mono text-zinc-500 uppercase">Secret Configured</span>
              <div className="font-semibold text-emerald-400 flex items-center space-x-1">
                <ShieldCheck className="w-3.5 h-3.5" />
                <span>Encrypted AES-256</span>
              </div>
            </div>
          </div>
        </div>

        {/* Quick Actions & Subscription Fan-Out Summary */}
        <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 flex flex-col justify-between space-y-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-[11px] font-mono font-bold text-zinc-400 uppercase tracking-wider">
                Active Subscriptions
              </span>
              <span className="text-xs font-mono font-bold text-emerald-400">
                {subscriptions.length} connected
              </span>
            </div>
            <p className="text-xs text-zinc-400 leading-relaxed">
              Events arriving at this source are matched and forwarded to all connected destinations.
            </p>
          </div>

          <button
            onClick={() => navigate('/subscriptions')}
            className="w-full py-2.5 px-3 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-200 hover:text-white text-xs font-semibold flex items-center justify-center space-x-2 transition-colors"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>Connect New Destination</span>
          </button>
        </div>
      </div>

      {/* Cryptographic Verification Log */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h3 className="text-sm font-bold text-white tracking-tight">
              Cryptographic Verification Activity
            </h3>
            <p className="text-xs text-zinc-400">
              Audit log of HMAC-SHA256 signature verifications and timestamp defenses.
            </p>
          </div>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                <th className="py-2.5 px-3">Received At</th>
                <th className="py-2.5 px-3">External Event ID</th>
                <th className="py-2.5 px-3">Signature Verification</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 font-mono">
              {logs.length === 0 ? (
                <tr>
                  <td colSpan={3} className="py-6 text-center text-zinc-500 italic">
                    No webhook signatures received yet.
                  </td>
                </tr>
              ) : (
                logs.map((log, idx) => (
                  <tr key={idx} className="hover:bg-zinc-900/40 transition-colors">
                    <td className="py-2.5 px-3 text-zinc-300">
                      {new Date(log.received_at).toLocaleString()}
                    </td>
                    <td className="py-2.5 px-3 text-zinc-400">
                      {log.external_event_id || '—'}
                    </td>
                    <td className="py-2.5 px-3">
                      {log.signature_valid ? (
                        <span className="inline-flex items-center space-x-1 px-2 py-0.5 rounded-full text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-semibold">
                          <ShieldCheck className="w-3 h-3" />
                          <span>Valid Constant-Time Match</span>
                        </span>
                      ) : (
                        <span className="inline-flex items-center space-x-1 px-2 py-0.5 rounded-full text-[10px] bg-rose-500/10 text-rose-400 border border-rose-500/20 font-semibold">
                          <ShieldAlert className="w-3 h-3" />
                          <span>Signature Failed (Rejected)</span>
                        </span>
                      )}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Connected Subscriptions Matrix */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        <h3 className="text-sm font-bold text-white tracking-tight">Connected Routing Subscriptions</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {subscriptions.length === 0 ? (
            <div className="col-span-2 text-xs text-zinc-500 italic py-4 text-center">
              No subscriptions configured for this source yet.
            </div>
          ) : (
            subscriptions.map((sub) => (
              <div
                key={sub.id}
                onClick={() => navigate(`/subscriptions/${sub.id}`)}
                className="p-4 rounded-xl bg-zinc-950 hover:bg-zinc-900/60 border border-zinc-800 cursor-pointer transition-all space-y-2 group"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <Zap className="w-4 h-4 text-purple-400" />
                    <span className="text-xs font-bold text-white group-hover:text-emerald-400 transition-colors">
                      {sub.destination_name || 'Forwarding Destination'}
                    </span>
                  </div>
                  <span
                    className={`px-2 py-0.5 rounded-full text-[10px] font-mono font-semibold uppercase ${
                      sub.is_active
                        ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                        : 'bg-zinc-800 text-zinc-400'
                    }`}
                  >
                    {sub.is_active ? 'Active' : 'Paused'}
                  </span>
                </div>
                <div className="flex flex-wrap gap-1">
                  {sub.event_types.map((et) => (
                    <span
                      key={et}
                      className="px-1.5 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-[10px] font-mono text-zinc-300"
                    >
                      {et}
                    </span>
                  ))}
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Rotate Secret Confirmation Modal */}
      <ConfirmModal
        isOpen={isRotateConfirmOpen}
        onClose={() => setIsRotateConfirmOpen(false)}
        onConfirm={handleRotateSecret}
        title="Rotate Inbound Signing Secret?"
        description="Rotating this secret will generate a new cryptographic signing key. Any webhooks dispatched with the old secret will fail signature validation until your provider settings are updated."
        confirmText="Rotate Secret"
        variant="warning"
        isLoading={isRotating}
      />

      {/* Delete Source Confirmation Modal */}
      <ConfirmModal
        isOpen={isDeleteConfirmOpen}
        onClose={() => setIsDeleteConfirmOpen(false)}
        onConfirm={handleDeleteSource}
        title="Delete Inbound Source?"
        description="This will permanently delete this source and remove its public ingestion webhook endpoint. Associated subscriptions and historical event records will remain accessible."
        confirmText="Delete Source"
        variant="danger"
        isLoading={isDeleting}
      />

      {/* Dedicated One-Time Secret Reveal Modal */}
      {newSecret && (
        <SecretRevealModal
          isOpen={!!newSecret}
          onClose={() => setNewSecret(null)}
          title="New Signing Secret Generated"
          subtitle={`Copy this secret and configure it in your ${source.name} webhook settings.`}
          secret={newSecret}
        />
      )}
    </div>
  );
};
