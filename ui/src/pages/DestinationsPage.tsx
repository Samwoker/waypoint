import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Send,
  Plus,
  ArrowRight,
  Search,
  Zap,
  Activity,
  Pause,
  Play,
  Trash2,
  CheckCircle2,
  XCircle,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  fetchDestinationsRequest,
  createDestinationRequest,
} from '../store/slices/destinationsSlice';
import { api } from '../api/client';
import { Destination } from '../types';
import { useToast } from '../context/ToastContext';
import { ConfirmModal } from '../components/common/ConfirmModal';
import { EmptyState, TableSkeleton } from '../components/common/Skeleton';

export const DestinationsPage: React.FC = () => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const toast = useToast();
  const { destinations, isLoading } = useAppSelector((state) => state.destinations);

  const [searchQuery, setSearchQuery] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [deleteDestId, setDeleteDestId] = useState<string | null>(null);

  // Form State
  const [name, setName] = useState('');
  const [url, setUrl] = useState('');
  const [description, setDescription] = useState('');
  const [timeoutMs, setTimeoutMs] = useState(5000);
  const [maxRetries, setMaxRetries] = useState(5);
  const [rateLimitRps, setRateLimitRps] = useState(100);

  useEffect(() => {
    dispatch(fetchDestinationsRequest());
  }, [dispatch]);

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name || !url) {
      toast.error('Validation Error', 'Name and URL are required.');
      return;
    }

    dispatch(
      createDestinationRequest({
        name,
        url,
        description: description || undefined,
        timeout_ms: timeoutMs,
        max_retries: maxRetries,
        rate_limit_rps: rateLimitRps,
      })
    );

    setIsCreateModalOpen(false);
    toast.success('Destination Created', 'Target webhook endpoint registered.');
    setName('');
    setUrl('');
    setDescription('');
  };

  const handleToggleStatus = async (e: React.MouseEvent, dest: Destination) => {
    e.stopPropagation();
    try {
      if (dest.is_active) {
        await api.pauseDestination(dest.id);
        toast.warning('Destination Paused', `${dest.name} will not receive forwarded webhooks.`);
      } else {
        await api.resumeDestination(dest.id);
        toast.success('Destination Resumed', `${dest.name} is active.`);
      }
      dispatch(fetchDestinationsRequest());
    } catch (err: any) {
      toast.error('Failed to update status', err.message);
    }
  };

  const filteredDestinations = destinations.filter(
    (d) =>
      d.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      d.url.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-extrabold text-white tracking-tight">
            Target Destinations
          </h1>
          <p className="text-xs text-zinc-400 mt-1">
            Downstream HTTP endpoints, automated circuit breakers, rate limits, and retry policies.
          </p>
        </div>
        <button
          onClick={() => setIsCreateModalOpen(true)}
          className="px-4 py-2.5 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs transition-all shadow-md flex items-center space-x-2 active:scale-95 self-start sm:self-auto"
        >
          <Plus className="w-4 h-4" />
          <span>Create Destination</span>
        </button>
      </div>

      {/* Filter & Search Bar */}
      <div className="flex items-center space-x-3 bg-zinc-950 p-2 rounded-2xl border border-zinc-800">
        <div className="relative flex-1">
          <Search className="w-4 h-4 text-zinc-500 absolute left-3.5 top-3" />
          <input
            type="text"
            placeholder="Search destinations by name or endpoint URL..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-transparent pl-10 pr-4 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none"
          />
        </div>
        <span className="text-xs font-mono text-zinc-500 px-3">
          {filteredDestinations.length} endpoints
        </span>
      </div>

      {/* Destinations Table */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        {isLoading ? (
          <TableSkeleton rows={5} cols={5} />
        ) : filteredDestinations.length === 0 ? (
          <EmptyState
            icon={Send}
            title="No destination endpoints configured"
            description="Register your API servers or third-party webhooks to receive forwarded events from active subscriptions."
            actionText="Create Destination"
            onAction={() => setIsCreateModalOpen(true)}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                  <th className="py-3 px-3">Destination Name</th>
                  <th className="py-3 px-3">Target Endpoint URL</th>
                  <th className="py-3 px-3">Circuit State</th>
                  <th className="py-3 px-3">Retry Policy</th>
                  <th className="py-3 px-3">Status</th>
                  <th className="py-3 px-3 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/60 font-mono">
                {filteredDestinations.map((dest) => {
                  const isCircuitOpen = dest.status === 'circuit_open';

                  return (
                    <tr
                      key={dest.id}
                      onClick={() => navigate(`/destinations/${dest.id}`)}
                      className="hover:bg-zinc-900/40 cursor-pointer transition-colors group"
                    >
                      <td className="py-3.5 px-3">
                        <div className="font-bold text-white font-sans group-hover:text-emerald-400 transition-colors flex items-center space-x-2">
                          <Send className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                          <span>{dest.name}</span>
                        </div>
                      </td>

                      <td className="py-3.5 px-3 text-zinc-300 font-mono truncate max-w-xs select-all">
                        {dest.url}
                      </td>

                      <td className="py-3.5 px-3">
                        <div className="flex items-center space-x-1.5 font-semibold">
                          <span
                            className={`w-2 h-2 rounded-full ${
                              isCircuitOpen ? 'bg-rose-400 animate-ping' : 'bg-emerald-400'
                            }`}
                          />
                          <span className={isCircuitOpen ? 'text-rose-400' : 'text-emerald-400'}>
                            {isCircuitOpen ? 'Circuit Open' : 'Closed (Healthy)'}
                          </span>
                        </div>
                      </td>

                      <td className="py-3.5 px-3 text-zinc-400">
                        {dest.max_retries} retries ({dest.timeout_ms}ms)
                      </td>

                      <td className="py-3.5 px-3">
                        <button
                          onClick={(e) => handleToggleStatus(e, dest)}
                          className={`px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase transition-colors ${
                            dest.is_active
                              ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 hover:bg-emerald-500/20'
                              : 'bg-amber-500/10 text-amber-400 border border-amber-500/20 hover:bg-amber-500/20'
                          }`}
                        >
                          {dest.is_active ? 'Active' : 'Paused'}
                        </button>
                      </td>

                      <td className="py-3.5 px-3 text-right">
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            navigate(`/destinations/${dest.id}`);
                          }}
                          className="px-2.5 py-1 rounded-lg bg-zinc-900 hover:bg-zinc-800 text-zinc-300 text-xs font-semibold inline-flex items-center space-x-1 transition-colors"
                        >
                          <span>Manage</span>
                          <ArrowRight className="w-3 h-3" />
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

      {/* Create Destination Modal */}
      {isCreateModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-150">
          <div className="bg-[#121215] border border-zinc-800 rounded-3xl max-w-md w-full p-6 shadow-2xl space-y-5">
            <div className="flex items-center space-x-2.5">
              <div className="p-2 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                <Send className="w-5 h-5" />
              </div>
              <div>
                <h3 className="text-base font-bold text-white">Create Target Destination</h3>
                <p className="text-xs text-zinc-400">Register outbound receiver endpoint</p>
              </div>
            </div>

            <form onSubmit={handleCreate} className="space-y-4 text-xs font-mono">
              <div className="space-y-1">
                <label className="text-zinc-400">Destination Name</label>
                <input
                  type="text"
                  required
                  placeholder="Billing API Receiver"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                />
              </div>

              <div className="space-y-1">
                <label className="text-zinc-400">Endpoint URL (HTTPS Preferred)</label>
                <input
                  type="url"
                  required
                  placeholder="https://api.example.com/webhooks"
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-emerald-400 focus:outline-none focus:border-zinc-600 font-mono text-xs"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <label className="text-zinc-400">Timeout (ms)</label>
                  <input
                    type="number"
                    value={timeoutMs}
                    onChange={(e) => setTimeoutMs(Number(e.target.value))}
                    className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                  />
                </div>

                <div className="space-y-1">
                  <label className="text-zinc-400">Max Retries</label>
                  <input
                    type="number"
                    value={maxRetries}
                    onChange={(e) => setMaxRetries(Number(e.target.value))}
                    className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                  />
                </div>
              </div>

              <div className="space-y-1">
                <label className="text-zinc-400">Rate Limit (req/sec)</label>
                <input
                  type="number"
                  value={rateLimitRps}
                  onChange={(e) => setRateLimitRps(Number(e.target.value))}
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
                  Register Endpoint
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
