import React, { useEffect, useState } from 'react';
import {
  CheckCircle2,
  Cpu,
  Plus,
  RotateCcw,
  Trash2,
  X,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  createDestinationRequest,
  deleteDestinationRequest,
  fetchDestinationsRequest,
  resetCircuitRequest,
} from '../store/slices/destinationsSlice';

export const DestinationsPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const { destinations, isLoading } = useAppSelector((state) => state.destinations);
  const [isCreateOpen, setIsCreateOpen] = useState(false);

  // Form states
  const [name, setName] = useState('');
  const [url, setUrl] = useState('');
  const [timeoutMs, setTimeoutMs] = useState(5000);
  const [maxRetries, setMaxRetries] = useState(5);
  const [rateLimit, setRateLimit] = useState<number | undefined>(undefined);

  useEffect(() => {
    dispatch(fetchDestinationsRequest());
  }, [dispatch]);

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    dispatch(
      createDestinationRequest({
        name,
        url,
        timeout_ms: timeoutMs,
        max_retry_count: maxRetries,
        rate_limit: rateLimit,
      })
    );
    setIsCreateOpen(false);
    setName('');
    setUrl('');
  };

  const handleResetCircuit = (id: string) => {
    dispatch(resetCircuitRequest(id));
  };

  const handleDelete = (id: string) => {
    if (!confirm('Are you sure you want to delete this destination endpoint?')) return;
    dispatch(deleteDestinationRequest(id));
  };

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-white tracking-tight">Destination Endpoints & Circuit Breakers</h1>
          <p className="text-xs text-zinc-400">Configure target URLs, retry policies, timeouts, and monitor automated circuit breakers.</p>
        </div>
        <button
          onClick={() => setIsCreateOpen(true)}
          className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-white text-zinc-950 font-semibold text-xs hover:bg-zinc-200 transition-colors shadow-md"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>Connect Destination</span>
        </button>
      </div>

      {/* Destinations Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
        {destinations.map((dest) => (
          <div
            key={dest.id}
            className="p-5 rounded-2xl bg-[#121215] border border-zinc-800 hover:border-zinc-700 transition-all space-y-4 shadow-lg flex flex-col justify-between"
          >
            <div>
              <div className="flex items-start justify-between">
                <div className="flex items-center space-x-2.5">
                  <div className="p-2 rounded-lg bg-zinc-900 border border-zinc-800 text-blue-400">
                    <Cpu className="w-4 h-4" />
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold text-white">{dest.name}</h3>
                    <span className="text-[10px] font-mono text-zinc-500">ID: {dest.id.slice(0, 8)}...</span>
                  </div>
                </div>

                <span
                  className={`text-[10px] font-mono px-2 py-0.5 rounded-full border font-medium ${
                    dest.circuit_status === 'closed'
                      ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                      : dest.circuit_status === 'half_open'
                      ? 'bg-amber-500/10 text-amber-400 border-amber-500/20'
                      : 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                  }`}
                >
                  Circuit: {dest.circuit_status.toUpperCase()}
                </span>
              </div>

              {/* URL bar */}
              <div className="mt-4 p-2.5 rounded-lg bg-zinc-950 border border-zinc-800 text-xs font-mono text-zinc-300 truncate">
                {dest.url}
              </div>

              {/* Policy badges */}
              <div className="mt-3 text-xs text-zinc-400 space-y-1.5 font-mono">
                <div className="flex justify-between">
                  <span className="text-zinc-500">Max Attempts:</span>
                  <span className="text-zinc-300">{dest.max_retry_count} retries</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-500">Timeout Budget:</span>
                  <span className="text-zinc-300">{dest.timeout_ms} ms</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-500">Failures:</span>
                  <span className={dest.consecutive_failures > 0 ? 'text-rose-400 font-semibold' : 'text-zinc-400'}>
                    {dest.consecutive_failures} consecutive
                  </span>
                </div>
              </div>
            </div>

            <div className="pt-3 border-t border-zinc-800/80 flex items-center justify-between text-xs">
              {dest.circuit_status !== 'closed' ? (
                <button
                  onClick={() => handleResetCircuit(dest.id)}
                  className="flex items-center space-x-1 text-xs text-amber-400 hover:text-amber-300 font-medium"
                >
                  <RotateCcw className="w-3.5 h-3.5" />
                  <span>Reset Circuit</span>
                </button>
              ) : (
                <span className="text-[11px] text-emerald-400 flex items-center space-x-1">
                  <CheckCircle2 className="w-3.5 h-3.5" />
                  <span>Healthy & Passing</span>
                </span>
              )}

              <button
                onClick={() => handleDelete(dest.id)}
                className="p-1 rounded text-zinc-500 hover:text-rose-400 transition-colors"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Connect Destination Modal */}
      {isCreateOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in">
          <div className="w-full max-w-lg bg-[#121215] border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden">
            <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800 bg-zinc-900/40">
              <h3 className="text-sm font-semibold text-white">Connect Destination Endpoint</h3>
              <button onClick={() => setIsCreateOpen(false)} className="text-zinc-400 hover:text-white">
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleCreate} className="p-6 space-y-4">
              <div>
                <label className="block text-xs font-mono text-zinc-400 mb-1">Destination Name</label>
                <input
                  type="text"
                  required
                  placeholder="Order Processing Service"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                />
              </div>

              <div>
                <label className="block text-xs font-mono text-zinc-400 mb-1">Target Endpoint URL</label>
                <input
                  type="url"
                  required
                  placeholder="https://api.example.com/webhooks/receiver"
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-mono text-zinc-400 mb-1">Timeout (ms)</label>
                  <input
                    type="number"
                    min={500}
                    max={60000}
                    value={timeoutMs}
                    onChange={(e) => setTimeoutMs(Number(e.target.value))}
                    className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                  />
                </div>

                <div>
                  <label className="block text-xs font-mono text-zinc-400 mb-1">Max Retry Count</label>
                  <input
                    type="number"
                    min={1}
                    max={20}
                    value={maxRetries}
                    onChange={(e) => setMaxRetries(Number(e.target.value))}
                    className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                  />
                </div>
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
                  Save Destination
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
