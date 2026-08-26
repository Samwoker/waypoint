import React, { useEffect, useState } from 'react';
import {
  ArrowRight,
  Cpu,
  Plus,
  Radio,
  Trash2,
  X,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchDestinationsRequest } from '../store/slices/destinationsSlice';
import { fetchSourcesRequest } from '../store/slices/sourcesSlice';
import {
  createSubscriptionRequest,
  deleteSubscriptionRequest,
  fetchSubscriptionsRequest,
  toggleSubscriptionRequest,
} from '../store/slices/subscriptionsSlice';
import { Subscription } from '../types';

export const SubscriptionsPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const { subscriptions, isLoading } = useAppSelector((state) => state.subscriptions);
  const { sources } = useAppSelector((state) => state.sources);
  const { destinations } = useAppSelector((state) => state.destinations);
  const [isCreateOpen, setIsCreateOpen] = useState(false);

  // Form states
  const [sourceId, setSourceId] = useState('');
  const [destinationId, setDestinationId] = useState('');
  const [eventTypesInput, setEventTypesInput] = useState('*');
  const [filterExpression, setFilterExpression] = useState('');

  useEffect(() => {
    dispatch(fetchSubscriptionsRequest());
    dispatch(fetchSourcesRequest());
    dispatch(fetchDestinationsRequest());
  }, [dispatch]);

  useEffect(() => {
    if (sources.length > 0 && !sourceId) setSourceId(sources[0].id);
    if (destinations.length > 0 && !destinationId) setDestinationId(destinations[0].id);
  }, [sources, destinations]);

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    const types = eventTypesInput.split(',').map((t) => t.trim()).filter(Boolean);
    dispatch(
      createSubscriptionRequest({
        source_id: sourceId,
        destination_id: destinationId,
        event_types: types.length > 0 ? types : ['*'],
        filter_expression: filterExpression || undefined,
      })
    );
    setIsCreateOpen(false);
    setFilterExpression('');
  };

  const handleDelete = (id: string) => {
    if (!confirm('Are you sure you want to remove this subscription rule?')) return;
    dispatch(deleteSubscriptionRequest(id));
  };

  const handleToggle = (sub: Subscription) => {
    dispatch(toggleSubscriptionRequest({ id: sub.id, is_active: !sub.is_active }));
  };

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6 animate-in fade-in duration-150">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-white tracking-tight">Subscriptions & Routing Rules</h1>
          <p className="text-xs text-zinc-400">Map inbound webhook sources to target destinations with event-type filters and transformations.</p>
        </div>
        <button
          onClick={() => setIsCreateOpen(true)}
          className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-white text-zinc-950 font-semibold text-xs hover:bg-zinc-200 transition-colors shadow-md"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>New Routing Rule</span>
        </button>
      </div>

      {/* Subscriptions Table */}
      <div className="rounded-2xl border border-zinc-800 bg-[#121215] overflow-hidden shadow-xl">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-zinc-900/80 border-b border-zinc-800 text-zinc-400">
              <tr>
                <th className="px-5 py-3 font-semibold">Source</th>
                <th className="px-5 py-3 font-semibold">Routing</th>
                <th className="px-5 py-3 font-semibold">Destination</th>
                <th className="px-5 py-3 font-semibold">Event Filter</th>
                <th className="px-5 py-3 font-semibold">Status</th>
                <th className="px-5 py-3 font-semibold text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 text-zinc-300">
              {subscriptions.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-5 py-10 text-center text-zinc-500 font-sans">
                    No subscriptions configured. Click "New Routing Rule" to connect a source to a destination.
                  </td>
                </tr>
              ) : (
                subscriptions.map((sub) => (
                  <tr key={sub.id} className="hover:bg-zinc-900/50 transition-colors">
                    <td className="px-5 py-3.5 font-semibold text-white">
                      <div className="flex items-center space-x-2">
                        <Radio className="w-3.5 h-3.5 text-emerald-400" />
                        <span>{sub.source_name || sub.source_id.slice(0, 8)}</span>
                      </div>
                    </td>
                    <td className="px-5 py-3.5 text-zinc-500">
                      <ArrowRight className="w-4 h-4 text-zinc-600" />
                    </td>
                    <td className="px-5 py-3.5 font-semibold text-white">
                      <div className="flex items-center space-x-2">
                        <Cpu className="w-3.5 h-3.5 text-blue-400" />
                        <span>{sub.destination_name || sub.destination_id.slice(0, 8)}</span>
                      </div>
                    </td>
                    <td className="px-5 py-3.5">
                      <div className="flex flex-wrap gap-1">
                        {sub.event_types.map((type) => (
                          <span
                            key={type}
                            className="px-2 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-[10px] text-zinc-300 font-mono"
                          >
                            {type}
                          </span>
                        ))}
                      </div>
                    </td>
                    <td className="px-5 py-3.5">
                      <button
                        onClick={() => handleToggle(sub)}
                        className={`px-2 py-0.5 rounded-full text-[10px] font-medium border transition-colors ${
                          sub.is_active
                            ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                            : 'bg-zinc-800 text-zinc-500 border-zinc-700'
                        }`}
                      >
                        {sub.is_active ? 'ACTIVE' : 'PAUSED'}
                      </button>
                    </td>
                    <td className="px-5 py-3.5 text-right">
                      <button
                        onClick={() => handleDelete(sub.id)}
                        className="p-1 rounded text-zinc-500 hover:text-rose-400 transition-colors"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Create Subscription Modal */}
      {isCreateOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in">
          <div className="w-full max-w-lg bg-[#121215] border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden">
            <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800 bg-zinc-900/40">
              <h3 className="text-sm font-semibold text-white">Create Subscription & Routing Rule</h3>
              <button onClick={() => setIsCreateOpen(false)} className="text-zinc-400 hover:text-white">
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleCreate} className="p-6 space-y-4">
              <div>
                <label className="block text-xs font-mono text-zinc-400 mb-1">Source Webhook</label>
                <select
                  value={sourceId}
                  onChange={(e) => setSourceId(e.target.value)}
                  className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                >
                  {sources.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.name} (/hooks/{s.slug})
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-mono text-zinc-400 mb-1">Target Destination</label>
                <select
                  value={destinationId}
                  onChange={(e) => setDestinationId(e.target.value)}
                  className="w-full px-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-lg focus:outline-none focus:border-zinc-600"
                >
                  {destinations.map((d) => (
                    <option key={d.id} value={d.id}>
                      {d.name} ({d.url})
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-mono text-zinc-400 mb-1">Event Type Filter (comma separated or *)</label>
                <input
                  type="text"
                  placeholder="payment.succeeded, order.created, *"
                  value={eventTypesInput}
                  onChange={(e) => setEventTypesInput(e.target.value)}
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
                  Create Rule
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
