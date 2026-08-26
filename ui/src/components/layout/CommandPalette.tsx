import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Activity,
  BookOpen,
  Code2,
  Cpu,
  Key,
  Radio,
  RefreshCw,
  Search,
  Send,
  SlidersHorizontal,
  Zap,
} from 'lucide-react';

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
}) => {
  const navigate = useNavigate();
  const [query, setQuery] = useState('');

  const commands = [
    {
      group: 'OBSERVE',
      items: [
        { path: '/', label: 'Overview & Telemetry', icon: Activity, desc: 'System KPIs, throughput & rates' },
        { path: '/events', label: 'Live Events Stream', icon: Zap, desc: 'Real-time payload stream & headers' },
        { path: '/deliveries', label: 'Deliveries & Traces', icon: Send, desc: 'HTTP attempt status & 1-click retry' },
        { path: '/dlq', label: 'Dead Letter Queue', icon: RefreshCw, desc: 'Quarantined failures & bulk replay' },
      ],
    },
    {
      group: 'BUILD & ROUTE',
      items: [
        { path: '/sources', label: 'Inbound Sources', icon: Radio, desc: 'Webhook URLs & signing secrets' },
        { path: '/destinations', label: 'Destinations & Circuit Breakers', icon: Cpu, desc: 'Downstream HTTP endpoints' },
        { path: '/subscriptions', label: 'Subscriptions & Rules', icon: SlidersHorizontal, desc: 'Route sources to destinations' },
        { path: '/transformations', label: 'Transformation Studio', icon: Code2, desc: 'JSONPath mapping & dry-run test' },
      ],
    },
    {
      group: 'DEVELOPER & MANAGE',
      items: [
        { path: '/docs', label: 'Documentation & API Reference', icon: BookOpen, desc: 'cURL, TypeScript, Python examples' },
        { path: '/apikeys', label: 'API Keys & Access Tokens', icon: Key, desc: 'Manage credentials & rate limits' },
      ],
    },
  ];

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        if (isOpen) onClose();
      }
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const filteredGroups = commands
    .map((group) => ({
      ...group,
      items: group.items.filter(
        (item) =>
          item.label.toLowerCase().includes(query.toLowerCase()) ||
          item.desc.toLowerCase().includes(query.toLowerCase())
      ),
    }))
    .filter((group) => group.items.length > 0);

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-100">
      <div className="w-full max-w-xl bg-[#121215] border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col">
        {/* Search Input Bar */}
        <div className="flex items-center px-4 py-3.5 border-b border-zinc-800/80 bg-zinc-900/50">
          <Search className="w-4 h-4 text-zinc-500 mr-3 shrink-0" />
          <input
            autoFocus
            type="text"
            placeholder="Search commands, destinations, sources, docs... (Esc to close)"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="w-full bg-transparent text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none font-mono"
          />
          <kbd className="px-2 py-0.5 text-[10px] font-mono bg-zinc-950 border border-zinc-800 text-zinc-500 rounded">
            ESC
          </kbd>
        </div>

        {/* Results List */}
        <div className="p-2 max-h-80 overflow-y-auto space-y-3">
          {filteredGroups.length === 0 ? (
            <div className="p-6 text-center text-xs text-zinc-500 font-mono">
              No matching pages or actions found.
            </div>
          ) : (
            filteredGroups.map((group) => (
              <div key={group.group} className="space-y-1">
                <div className="px-3 py-1 text-[10px] font-mono text-zinc-500 uppercase tracking-wider">
                  {group.group}
                </div>
                {group.items.map((item) => {
                  const Icon = item.icon;
                  return (
                    <button
                      key={item.path}
                      onClick={() => {
                        navigate(item.path);
                        onClose();
                      }}
                      className="w-full flex items-center justify-between p-2.5 rounded-xl hover:bg-zinc-800/80 text-left transition-colors group"
                    >
                      <div className="flex items-center space-x-3">
                        <div className="p-2 rounded-lg bg-zinc-900 border border-zinc-800 text-zinc-400 group-hover:text-white group-hover:border-zinc-700 transition-colors">
                          <Icon className="w-4 h-4" />
                        </div>
                        <div>
                          <div className="text-xs font-semibold text-zinc-200 group-hover:text-white">
                            {item.label}
                          </div>
                          <div className="text-[10px] text-zinc-500 font-mono">{item.desc}</div>
                        </div>
                      </div>
                      <span className="text-[10px] font-mono text-zinc-500 group-hover:text-zinc-300">
                        Jump →
                      </span>
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
