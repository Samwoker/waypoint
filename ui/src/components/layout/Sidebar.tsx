import React from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import {
  Activity,
  BookOpen,
  Code2,
  Cpu,
  Key,
  Layers,
  Radio,
  RefreshCw,
  Search,
  Send,
  SlidersHorizontal,
  Zap,
} from 'lucide-react';

interface SidebarProps {
  onOpenCommandPalette: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ onOpenCommandPalette }) => {
  const navigate = useNavigate();
  const location = useLocation();

  const sections = [
    {
      title: 'OBSERVE',
      items: [
        { path: '/', label: 'Overview & Telemetry', icon: Activity },
        { path: '/events', label: 'Live Events', icon: Zap, badge: 'Live' },
        { path: '/deliveries', label: 'Deliveries', icon: Send },
        { path: '/dlq', label: 'Dead Letter Queue', icon: RefreshCw },
      ],
    },
    {
      title: 'BUILD & ROUTE',
      items: [
        { path: '/sources', label: 'Inbound Sources', icon: Radio },
        { path: '/destinations', label: 'Destinations', icon: Cpu },
        { path: '/subscriptions', label: 'Subscriptions', icon: SlidersHorizontal },
        { path: '/transformations', label: 'Transformation Studio', icon: Code2 },
      ],
    },
    {
      title: 'DOCUMENTATION',
      items: [
        { path: '/docs', label: 'Guides & API Reference', icon: BookOpen },
      ],
    },
    {
      title: 'MANAGE',
      items: [
        { path: '/apikeys', label: 'API Keys & Tokens', icon: Key },
      ],
    },
  ];

  const isCurrentActive = (path: string) => {
    if (path === '/') return location.pathname === '/';
    return location.pathname.startsWith(path);
  };

  return (
    <aside className="w-64 border-r border-zinc-800 bg-[#0c0c0e] flex flex-col justify-between shrink-0 h-screen sticky top-0">
      {/* Brand Header */}
      <div>
        <div
          onClick={() => navigate('/')}
          className="h-16 flex items-center justify-between px-5 border-b border-zinc-800/80 bg-zinc-950/40 cursor-pointer"
        >
          <div className="flex items-center space-x-2.5">
            <div className="w-7 h-7 rounded-lg bg-gradient-to-tr from-zinc-800 to-zinc-700 border border-zinc-600 flex items-center justify-center shadow-inner">
              <Layers className="w-4 h-4 text-white" />
            </div>
            <div className="flex items-baseline space-x-1.5">
              <span className="font-bold text-sm tracking-tight text-white font-mono">WAYPOINT</span>
              <span className="text-[10px] text-zinc-500 font-mono">v0.1</span>
            </div>
          </div>
        </div>

        {/* Quick Search Shortcut Bar */}
        <div className="p-3">
          <button
            onClick={onOpenCommandPalette}
            className="w-full flex items-center justify-between px-3 py-2 text-xs text-zinc-400 bg-zinc-900/60 hover:bg-zinc-900 border border-zinc-800 rounded-lg transition-all"
          >
            <div className="flex items-center space-x-2">
              <Search className="w-3.5 h-3.5 text-zinc-500" />
              <span>Quick Search...</span>
            </div>
            <kbd className="px-1.5 py-0.5 text-[10px] font-mono bg-zinc-950 border border-zinc-800 text-zinc-400 rounded">
              Ctrl+K
            </kbd>
          </button>
        </div>

        {/* Navigation Sections */}
        <nav className="px-3 py-1 space-y-5 overflow-y-auto max-h-[calc(100vh-210px)]">
          {sections.map((section) => (
            <div key={section.title} className="space-y-1">
              <div className="px-2 text-[10px] font-mono font-semibold text-zinc-500 tracking-wider uppercase">
                {section.title}
              </div>
              <div className="space-y-0.5">
                {section.items.map((item) => {
                  const Icon = item.icon;
                  const isActive = isCurrentActive(item.path);
                  return (
                    <button
                      key={item.path}
                      onClick={() => navigate(item.path)}
                      className={`w-full flex items-center justify-between px-2.5 py-2 text-xs font-medium rounded-lg transition-all group ${
                        isActive
                          ? 'bg-zinc-800/90 text-white font-semibold shadow-sm border border-zinc-700/60'
                          : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/50'
                      }`}
                    >
                      <div className="flex items-center space-x-2.5">
                        <Icon
                          className={`w-4 h-4 transition-colors ${
                            isActive ? 'text-white' : 'text-zinc-500 group-hover:text-zinc-300'
                          }`}
                        />
                        <span>{item.label}</span>
                      </div>
                      {item.badge && (
                        <span className="text-[9px] font-mono px-1.5 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">
                          {item.badge}
                        </span>
                      )}
                    </button>
                  );
                })}
              </div>
            </div>
          ))}
        </nav>
      </div>

      {/* System Gateway Status Card (Bottom) */}
      <div className="p-3 border-t border-zinc-800/80 bg-zinc-950/60">
        <div className="p-2.5 rounded-lg bg-zinc-900/40 border border-zinc-800/80 text-xs">
          <div className="flex items-center justify-between mb-1">
            <span className="text-[11px] text-zinc-400 font-medium">Relay Gateway</span>
            <div className="flex items-center space-x-1.5">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
              <span className="text-[10px] font-mono text-emerald-400 font-medium">Healthy</span>
            </div>
          </div>
          <div className="text-[10px] font-mono text-zinc-500 flex justify-between">
            <span>Port: 3001</span>
            <span>Postgres + Redis</span>
          </div>
        </div>
      </div>
    </aside>
  );
};
