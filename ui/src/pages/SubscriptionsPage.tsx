import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Zap,
  Plus,
  ArrowRight,
  Search,
  Radio,
  Send,
  Trash2,
  Pause,
  Play,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  fetchSubscriptionsRequest,
  createSubscriptionRequest,
} from '../store/slices/subscriptionsSlice';
import { fetchSourcesRequest } from '../store/slices/sourcesSlice';
import { fetchDestinationsRequest } from '../store/slices/destinationsSlice';
import { api } from '../api/client';
import { Subscription } from '../types';
import { useToast } from '../context/ToastContext';
import { EmptyState, TableSkeleton } from '../components/common/Skeleton';

export const SubscriptionsPage: React.FC = () => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const toast = useToast();
  const { subscriptions, isLoading } = useAppSelector((state) => state.subscriptions);
  const { sources } = useAppSelector((state) => state.sources);
  const { destinations } = useAppSelector((state) => state.destinations);

  const [searchQuery, setSearchQuery] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);

  // Form State
  const [sourceId, setSourceId] = useState('');
  const [destinationId, setDestinationId] = useState('');
  const [eventTypesInput, setEventTypesInput] = useState('*');

  useEffect(() => {
    dispatch(fetchSubscriptionsRequest());
    dispatch(fetchSourcesRequest());
    dispatch(fetchDestinationsRequest());
  }, [dispatch]);

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    if (!sourceId || !destinationId) {
      toast.error('Validation Error', 'Source and Destination are required.');
      return;
    }

    const eventTypes = eventTypesInput
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s.length > 0);

    dispatch(
      createSubscriptionRequest({
        source_id: sourceId,
        destination_id: destinationId,
        event_types: eventTypes.length > 0 ? eventTypes : ['*'],
      })
    );

    setIsCreateModalOpen(false);
    toast.success('Subscription Created', 'Routing rule is active and forwarding webhooks.');
    setSourceId('');
    setDestinationId('');
    setEventTypesInput('*');
  };

  const filteredSubscriptions = subscriptions.filter(
    (sub) =>
      (sub.source_name || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
      (sub.destination_name || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
      sub.event_types.some((et) => et.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-extrabold text-white tracking-tight">
            Routing Subscriptions
          </h1>
          <p className="text-xs text-zinc-400 mt-1">
            Connect Inbound Sources to Target Destinations with event type wildcard filters and transformations.
          </p>
        </div>
        <button
          onClick={() => setIsCreateModalOpen(true)}
          className="px-4 py-2.5 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs transition-all shadow-md flex items-center space-x-2 active:scale-95 self-start sm:self-auto"
        >
          <Plus className="w-4 h-4" />
          <span>Connect Subscription</span>
        </button>
      </div>

      {/* Filter & Search Bar */}
      <div className="flex items-center space-x-3 bg-zinc-950 p-2 rounded-2xl border border-zinc-800">
        <div className="relative flex-1">
          <Search className="w-4 h-4 text-zinc-500 absolute left-3.5 top-3" />
          <input
            type="text"
            placeholder="Search subscriptions by source, destination, or event filter..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-transparent pl-10 pr-4 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none"
          />
        </div>
        <span className="text-xs font-mono text-zinc-500 px-3">
          {filteredSubscriptions.length} subscriptions
        </span>
      </div>

      {/* Subscriptions Table */}
      <div className="p-6 rounded-2xl bg-[#121215] border border-zinc-800 space-y-4">
        {isLoading ? (
          <TableSkeleton rows={5} cols={5} />
        ) : filteredSubscriptions.length === 0 ? (
          <EmptyState
            icon={Zap}
            title="No routing subscriptions yet"
            description="Create a subscription to link an Inbound Source to a Target Destination."
            actionText="Create Subscription"
            onAction={() => setIsCreateModalOpen(true)}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-zinc-800 text-zinc-500 font-mono text-[10px] uppercase">
                  <th className="py-3 px-3">Inbound Source</th>
                  <th className="py-3 px-3">Target Destination</th>
                  <th className="py-3 px-3">Event Filters</th>
                  <th className="py-3 px-3">Status</th>
                  <th className="py-3 px-3 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/60 font-mono">
                {filteredSubscriptions.map((sub) => (
                  <tr
                    key={sub.id}
                    onClick={() => navigate(`/subscriptions/${sub.id}`)}
                    className="hover:bg-zinc-900/40 cursor-pointer transition-colors group"
                  >
                    <td className="py-3.5 px-3">
                      <div className="font-bold text-white font-sans group-hover:text-emerald-400 transition-colors flex items-center space-x-2">
                        <Radio className="w-3.5 h-3.5 text-blue-400 shrink-0" />
                        <span>{sub.source_name || 'Inbound Source'}</span>
                      </div>
                    </td>

                    <td className="py-3.5 px-3">
                      <div className="font-bold text-white font-sans flex items-center space-x-2">
                        <Send className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                        <span>{sub.destination_name || 'Forwarding Destination'}</span>
                      </div>
                    </td>

                    <td className="py-3.5 px-3">
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
                    </td>

                    <td className="py-3.5 px-3">
                      <span
                        className={`px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase ${
                          sub.is_active
                            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                            : 'bg-zinc-800 text-zinc-400'
                        }`}
                      >
                        {sub.is_active ? 'Active' : 'Paused'}
                      </span>
                    </td>

                    <td className="py-3.5 px-3 text-right">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          navigate(`/subscriptions/${sub.id}`);
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

      {/* Create Subscription Modal */}
      {isCreateModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-150">
          <div className="bg-[#121215] border border-zinc-800 rounded-3xl max-w-md w-full p-6 shadow-2xl space-y-5">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2.5">
                <div className="p-2 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/20">
                  <Zap className="w-5 h-5" />
                </div>
                <div>
                  <h3 className="text-base font-bold text-white">Create Routing Subscription</h3>
                  <p className="text-xs text-zinc-400">Connect Source to Destination</p>
                </div>
              </div>
            </div>

            <form onSubmit={handleCreate} className="space-y-4 text-xs font-mono">
              <div className="space-y-1">
                <label className="text-zinc-400">Select Inbound Source</label>
                <select
                  required
                  value={sourceId}
                  onChange={(e) => setSourceId(e.target.value)}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                >
                  <option value="">-- Choose Inbound Source --</option>
                  {sources.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.name} (/hooks/{s.slug})
                    </option>
                  ))}
                </select>
              </div>

              <div className="space-y-1">
                <label className="text-zinc-400">Select Target Destination</label>
                <select
                  required
                  value={destinationId}
                  onChange={(e) => setDestinationId(e.target.value)}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-white focus:outline-none focus:border-zinc-600 font-mono text-xs"
                >
                  <option value="">-- Choose Target Endpoint --</option>
                  {destinations.map((d) => (
                    <option key={d.id} value={d.id}>
                      {d.name} ({d.url})
                    </option>
                  ))}
                </select>
              </div>

              <div className="space-y-1">
                <label className="text-zinc-400">
                  Event Types Filter (Comma-separated or * for all)
                </label>
                <input
                  type="text"
                  required
                  placeholder="payment.succeeded, charge.refunded"
                  value={eventTypesInput}
                  onChange={(e) => setEventTypesInput(e.target.value)}
                  className="w-full p-2.5 rounded-xl bg-zinc-950 border border-zinc-800 text-emerald-400 focus:outline-none focus:border-zinc-600 font-mono text-xs"
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
                  Create Subscription
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
