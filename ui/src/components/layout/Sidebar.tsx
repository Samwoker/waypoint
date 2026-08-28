import React from 'react';
import { useLocation, useNavigate, Link } from 'react-router-dom';
import {
  Activity,
  AlertTriangle,
  BarChart3,
  BookOpen,
  Code2,
  CreditCard,
  Gauge,
  Key,
  Layers,
  LayoutDashboard,
  LucideIcon,
  Radio,
  RefreshCw,
  Search,
  Send,
  Settings,
  Shield,
  Sparkles,
  Zap,
} from 'lucide-react';
import { useAppSelector } from '../../store/hooks';

interface SidebarProps {
  onOpenCommandPalette: () => void;
}

interface NavItem {
  label: string;
  icon: LucideIcon;
  path: string;
  aliases?: string[];
}

interface NavGroup {
  label: string;
  items: NavItem[];
}

export const Sidebar: React.FC<SidebarProps> = ({ onOpenCommandPalette }) => {
  const location = useLocation();
  const navigate = useNavigate();
  const { currentTenant, user } = useAppSelector((state) => state.auth);

  const navGroups: NavGroup[] = [
    {
      label: 'OVERVIEW',
      items: [
        {
          label: 'Dashboard',
          icon: LayoutDashboard,
          path: '/dashboard',
          aliases: ['/'],
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
          aliases: ['/dashboard/sources'],
        },
        {
          label: 'Events',
          icon: Layers,
          path: '/events',
          aliases: ['/dashboard/events'],
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
          aliases: ['/dashboard/destinations'],
        },
        {
          label: 'Subscriptions',
          icon: Zap,
          path: '/subscriptions',
          aliases: ['/dashboard/subscriptions'],
        },
        {
          label: 'Deliveries',
          icon: Activity,
          path: '/deliveries',
          aliases: ['/dashboard/deliveries'],
        },
        {
          label: 'Dead Letter Queue',
          icon: AlertTriangle,
          path: '/dlq',
          aliases: ['/dashboard/dlq'],
        },
      ],
    },
    {
      label: 'SUBSCRIPTION & USAGE',
      items: [
        {
          label: 'Usage & Quotas',
          icon: Gauge,
          path: '/usage',
          aliases: ['/dashboard/usage'],
        },
        {
          label: 'Billing & Plans',
          icon: CreditCard,
          path: '/billing',
          aliases: ['/dashboard/billing'],
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
          aliases: ['/dashboard/stats'],
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
          aliases: ['/dashboard/api-keys'],
        },
        {
          label: 'Transformations',
          icon: Code2,
          path: '/transformations',
          aliases: ['/dashboard/transformations'],
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
          label: 'Organization Settings',
          icon: Settings,
          path: '/settings',
          aliases: ['/dashboard/settings', '/tenants/settings', '/tenants'],
        },
      ],
    },
  ];

  return (
    <aside className="w-64 border-r border-zinc-800 bg-[#09090b] flex flex-col justify-between shrink-0 select-none">
      {/* Brand Header & Quick Command Palette */}
      <div className="p-4 space-y-4">
        <Link to="/dashboard" className="flex items-center space-x-2.5 px-2 py-1 group">
          <div className="p-2 rounded-xl bg-gradient-to-tr from-zinc-800 to-zinc-700 border border-zinc-600 shadow-sm group-hover:scale-105 transition-transform">
            <Layers className="w-4 h-4 text-white" />
          </div>
          <div>
            <div className="font-extrabold text-sm tracking-tight text-white flex items-center space-x-1.5">
              <span>RelayCore</span>
              <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                PaaS
              </span>
            </div>
            <div className="text-[10px] text-zinc-500 font-mono truncate max-w-[130px]">
              {currentTenant?.name || user?.email || 'Workspace'}
            </div>
          </div>
        </Link>

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
                  location.pathname === item.path ||
                  (item.aliases && item.aliases.includes(location.pathname)) ||
                  (item.path !== '/' && item.path !== '/dashboard' && location.pathname.startsWith(item.path));
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

      {/* Footer Plan Card */}
      <div className="p-3 border-t border-zinc-800/80 space-y-2">
        <div className="p-3 rounded-2xl bg-zinc-950 border border-zinc-800/80 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-[10px] font-mono font-bold uppercase text-zinc-400">
              Current Plan
            </span>
            <span className="px-1.5 py-0.5 rounded text-[10px] font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              FREE
            </span>
          </div>

          <div className="flex items-center justify-between text-xs pt-0.5">
            <span className="text-zinc-400 text-[11px]">25K Events / mo</span>
            <Link
              to="/billing"
              className="text-emerald-400 hover:text-emerald-300 font-semibold text-[11px] flex items-center space-x-0.5"
            >
              <span>Upgrade</span>
              <Sparkles className="w-3 h-3 ml-0.5" />
            </Link>
          </div>
        </div>
      </div>
    </aside>
  );
};
