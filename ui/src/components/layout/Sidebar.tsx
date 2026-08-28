import React from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import {
  Activity,
  AlertTriangle,
  BarChart3,
  BookOpen,
  Building,
  CheckCircle2,
  Code2,
  Cpu,
  Inbox,
  Key,
  Layers,
  LayoutDashboard,
  Radio,
  RefreshCw,
  Search,
  Send,
  Settings,
  Shield,
  Zap,
} from 'lucide-react';
import { useAppSelector } from '../../store/hooks';

interface SidebarProps {
  onOpenCommandPalette: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ onOpenCommandPalette }) => {
  const location = useLocation();
  const navigate = useNavigate();
  const { currentTenant } = useAppSelector((state) => state.auth);

  const navGroups = [
    {
      label: 'OVERVIEW',
      items: [
        {
          label: 'Dashboard',
          icon: LayoutDashboard,
          path: '/',
        },
      ],
    },
    {
      label: 'INGESTION',
      items: [
        {
          label: 'Sources',
          icon: Radio,
          path: '/sources',
        },
        {
          label: 'Events',
          icon: Layers,
          path: '/events',
        },
      ],
    },
    {
      label: 'DELIVERY',
      items: [
        {
          label: 'Destinations',
          icon: Send,
          path: '/destinations',
        },
        {
          label: 'Subscriptions',
          icon: Zap,
          path: '/subscriptions',
        },
        {
          label: 'Deliveries',
          icon: Activity,
          path: '/deliveries',
        },
        {
          label: 'Dead Letter Queue',
          icon: AlertTriangle,
          path: '/dlq',
        },
      ],
    },
    {
      label: 'OBSERVABILITY',
      items: [
        {
          label: 'Statistics',
          icon: BarChart3,
          path: '/stats',
        },
      ],
    },
    {
      label: 'DEVELOPER',
      items: [
        {
          label: 'API Keys',
          icon: Key,
          path: '/api-keys',
        },
        {
          label: 'Transformations',
          icon: Code2,
          path: '/transformations',
        },
        {
          label: 'Documentation',
          icon: BookOpen,
          path: '/docs',
        },
      ],
    },
    {
      label: 'ADMIN',
      items: [
        {
          label: 'Tenant Settings',
          icon: Settings,
          path: '/tenants/settings',
        },
      ],
    },
  ];

  return (
    <aside className="w-64 border-r border-zinc-800 bg-[#09090b] flex flex-col justify-between shrink-0 select-none">
      {/* Brand Header & Quick Command Palette */}
      <div className="p-4 space-y-4">
        <div className="flex items-center space-x-2.5 px-2 py-1">
          <div className="p-2 rounded-xl bg-gradient-to-tr from-zinc-800 to-zinc-700 border border-zinc-600 shadow-sm">
            <Layers className="w-4 h-4 text-white" />
          </div>
          <div>
            <div className="font-extrabold text-sm tracking-tight text-white flex items-center space-x-1.5">
              <span>RelayCore</span>
              <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                v1.0
              </span>
            </div>
            <div className="text-[10px] text-zinc-500 font-mono">
              {currentTenant ? currentTenant.name : 'Production Gateway'}
            </div>
          </div>
        </div>

        {/* Global Search / Command Palette shortcut button */}
        <button
          type="button"
          onClick={onOpenCommandPalette}
          className="w-full flex items-center justify-between px-3 py-2 text-xs text-zinc-400 bg-zinc-950/80 hover:bg-zinc-900 border border-zinc-800/80 rounded-xl transition-colors shadow-inner"
        >
          <div className="flex items-center space-x-2">
            <Search className="w-3.5 h-3.5 text-zinc-500" />
            <span className="text-[11px]">Quick search...</span>
          </div>
          <kbd className="px-1.5 py-0.5 text-[10px] font-mono bg-zinc-900 text-zinc-400 rounded border border-zinc-800">
            Ctrl+K
          </kbd>
        </button>
      </div>

      {/* Navigation Group Tree */}
      <div className="flex-1 overflow-y-auto px-3 space-y-5">
        {navGroups.map((group) => (
          <div key={group.label} className="space-y-1">
            <div className="px-2 text-[10px] font-mono font-semibold text-zinc-500 uppercase tracking-wider">
              {group.label}
            </div>
            <div className="space-y-0.5">
              {group.items.map((item) => {
                const isActive =
                  item.path === '/'
                    ? location.pathname === '/'
                    : location.pathname.startsWith(item.path);
                const Icon = item.icon;

                return (
                  <button
                    key={item.path}
                    type="button"
                    onClick={() => navigate(item.path)}
                    className={`w-full flex items-center space-x-2.5 px-2.5 py-2 rounded-xl text-xs font-medium transition-all ${
                      isActive
                        ? 'bg-zinc-800 text-white font-semibold border border-zinc-700/60 shadow-sm'
                        : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60'
                    }`}
                  >
                    <Icon
                      className={`w-4 h-4 ${
                        isActive ? 'text-emerald-400' : 'text-zinc-500'
                      }`}
                    />
                    <span>{item.label}</span>
                  </button>
                );
              })}
            </div>
          </div>
        ))}
      </div>

      {/* Footer System Status Badge */}
      <div className="p-4 border-t border-zinc-800/80">
        <div className="flex items-center justify-between p-2.5 rounded-xl bg-zinc-950 border border-zinc-800/80">
          <div className="flex items-center space-x-2">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
            <span className="text-[11px] font-mono font-semibold text-zinc-300">
              Gateway Active
            </span>
          </div>
          <span className="text-[10px] font-mono text-zinc-500">Postgres+Redis</span>
        </div>
      </div>
    </aside>
  );
};
