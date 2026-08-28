import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Radio,
  Plus,
  Copy,
  Check,
  ShieldCheck,
  RefreshCw,
  Trash2,
  ExternalLink,
  Search,
  Filter,
  Shield,
  Layers,
  ArrowRight,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  fetchSourcesRequest,
  createSourceRequest,
} from '../store/slices/sourcesSlice';
import { Source } from '../types';
import { useToast } from '../context/ToastContext';
import { SecretRevealModal } from '../components/common/SecretRevealModal';
import { ConfirmModal } from '../components/common/ConfirmModal';
import { EmptyState, TableSkeleton } from '../components/common/Skeleton';

export const SourcesPage: React.FC = () => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const toast = useToast();
  const { sources, isLoading } = useAppSelector((state) => state.sources);

  const [searchQuery, setSearchQuery] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [createdSecret, setCreatedSecret] = useState<{ name: string; secret: string } | null>(null);
  const [deleteSourceId, setDeleteSourceId] = useState<string | null>(null);

  // Form State
  const [name, setName] = useState('');
  const [slug, setSlug] = useState('');
  const [description, setDescription] = useState('');
  const [provider, setProvider] = useState('generic');
  const [verificationType, setVerificationType] = useState('generic_hmac');

  useEffect(() => {
    dispatch(fetchSourcesRequest());
  }, [dispatch]);

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name || !slug) {
      toast.error('Validation Error', 'Name and URL slug are required.');
      return;
    }

    dispatch(
      createSourceRequest({
        name,
        slug: slug.toLowerCase().replace(/[^a-z0-9-_]/g, '-'),
        description: description || undefined,
        provider,
        verification_type: verificationType,
      })
    );

    setIsCreateModalOpen(false);
    toast.success('Source Created', 'Inbound webhook endpoint is active.');
    // Reset form
    setName('');
    setSlug('');
    setDescription('');
  };

  const filteredSources = sources.filter(
    (s) =>
      s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      s.slug.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-extrabold text-white tracking-tight">Inbound Sources</h1>
          <p className="text-xs text-zinc-400 mt-1">
            Configure webhook entrypoints, cryptographic HMAC verification keys, and payload ingestion.
          </p>
        </div>
        <button
          onClick={() => setIsCreateModalOpen(true)}
          className="px-4 py-2.5 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs transition-all shadow-md flex items-center space-x-2 active:scale-95 self-start sm:self-auto"
        >
          <Plus className="w-4 h-4" />
          <span>Create Source</span>
        </button>
      </div>

      {/* Filter & Search Bar */}
      <div className="flex items-center space-x-3 bg-zinc-950 p-2 rounded-2xl border border-zinc-800">
        <div className="relative flex-1">
          <Search className="w-4 h-4 text-zinc-500 absolute left-3.5 top-3" />
          <input
            type="text"
            placeholder="Search sources by name or URL slug..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-transparent pl-10 pr-4 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none"
          />
        </div>
        <span className="text-xs font-mono text-zinc-500 px-3">
          {filteredSources.length} sources
        </span>
      </div>

      {/* Sources Table */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        {isLoading ? (
          <TableSkeleton rows={5} cols={5} />
        ) : filteredSources.length === 0 ? (
          <EmptyState
            icon={Radio}
            title="No webhook sources found"
            description="Create an inbound source to generate public webhook URLs for Stripe, GitHub, Shopify, or custom apps."
            actionText="Create your first source"
            onAction={() => setIsCreateModalOpen(true)}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                  <th className="py-3 px-3">Source Name</th>
                  <th className="py-3 px-3">Inbound URL Slug</th>
                  <th className="py-3 px-3">Provider & Verification</th>
                  <th className="py-3 px-3">Status</th>
                  <th className="py-3 px-3">Created</th>
                  <th className="py-3 px-3 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/60 font-mono">
                {filteredSources.map((source) => (
                  <tr
                    key={source.id}
                    onClick={() => navigate(`/sources/${source.id}`)}
                    className="hover:bg-zinc-900/40 cursor-pointer transition-colors group"
                  >
                    <td className="py-3.5 px-3">
                      <div className="font-bold text-white font-sans group-hover:text-emerald-400 transition-colors flex items-center space-x-2">
                        <Radio className="w-3.5 h-3.5 text-blue-400 shrink-0" />
                        <span>{source.name}</span>
                      </div>
                      {source.description && (
                        <div className="text-[11px] text-zinc-500 font-sans truncate max-w-xs">
                          {source.description}
                        </div>
                      )}
                    </td>

                    <td className="py-3.5 px-3 text-emerald-400 font-mono font-semibold">
                      /hooks/{source.slug}
                    </td>

                    <td className="py-3.5 px-3">
                      <div className="flex items-center space-x-1.5 text-zinc-300 capitalize font-sans">
                        <span className="px-1.5 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-[10px] font-mono">
                          {source.provider}
                        </span>
                        <span className="text-zinc-500">•</span>
                        <span className="text-zinc-400 text-[11px] font-mono">
                          {source.verification_type}
                        </span>
                      </div>
                    </td>

                    <td className="py-3.5 px-3">
                      <span
                        className={`px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase ${
                          source.is_active
                            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                            : 'bg-zinc-800 text-zinc-400'
                        }`}
                      >
                        {source.is_active ? 'Active' : 'Inactive'}
                      </span>
                    </td>

                    <td className="py-3.5 px-3 text-zinc-400">
                      {new Date(source.created_at).toLocaleDateString()}
                    </td>

                    <td className="py-3.5 px-3 text-right">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          navigate(`/sources/${source.id}`);
                        }}
                        className="px-2.5 py-1 rounded-lg bg-zinc-900 hover:bg-zinc-800 text-zinc-300 text-xs font-semibold inline-flex items-center space-x-1 transition-colors"
                      >
                        <span>Manage</span>
                        <ArrowRight className="w-3 h-3" />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Create Source Modal */}
      {isCreateModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-150">
          <div className="bg-[#121215] border border-zinc-800 rounded-3xl max-w-md w-full p-6 shadow-2xl space-y-5">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2.5">
                <div className="p-2 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                  <Radio className="w-5 h-5" />
                </div>
                <div>
                  <h3 className="text-base font-bold text-white">Create Inbound Source</h3>
                  <p className="text-xs text-zinc-400">Configure provider and cryptographic signing</p>
                </div>
              </div>
            </div>

            <form onSubmit={handleCreate} className="space-y-4 text-xs font-mono">
              <div className="space-y-1">
                <label className="text-zinc-400">Source Name</label>
                <input
                  type="text"
                  required
                  placeholder="Stripe Production"
                  value={name}
                  onChange={(e) => {
                    setName(e.target.value);
                    if (!slug) {
                      setSlug(e.target.value.toLowerCase().replace(/[^a-z0-9-_]/g, '-'));
                    }
                  }}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                />
              </div>

              <div className="space-y-1">
                <label className="text-zinc-400">Inbound URL Slug (/hooks/:slug)</label>
                <input
                  type="text"
                  required
                  placeholder="stripe-prod"
                  value={slug}
                  onChange={(e) => setSlug(e.target.value)}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-emerald-400 focus:outline-none focus:border-zinc-600 font-mono text-xs"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <label className="text-zinc-400">Provider</label>
                  <select
                    value={provider}
                    onChange={(e) => setProvider(e.target.value)}
                    className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                  >
                    <option value="generic">Generic</option>
                    <option value="stripe">Stripe</option>
                    <option value="github">GitHub</option>
                    <option value="shopify">Shopify</option>
                  </select>
                </div>

                <div className="space-y-1">
                  <label className="text-zinc-400">Verification Type</label>
                  <select
                    value={verificationType}
                    onChange={(e) => setVerificationType(e.target.value)}
                    className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                  >
                    <option value="generic_hmac">HMAC-SHA256</option>
                    <option value="stripe">Stripe v1 Header</option>
                    <option value="github">GitHub X-Hub Header</option>
                    <option value="none">None (Open)</option>
                  </select>
                </div>
              </div>

              <div className="space-y-1">
                <label className="text-zinc-400">Description (Optional)</label>
                <input
                  type="text"
                  placeholder="Inbound customer payment webhooks"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                />
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
                  Create Inbound Source
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
