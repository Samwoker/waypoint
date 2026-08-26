import React, { useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import {
  BookOpen,
  Building,
  Check,
  ChevronDown,
  LogOut,
  Send,
  User as UserIcon,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../../store/hooks';
import { logout, switchTenant } from '../../store/slices/authSlice';

interface HeaderProps {
  onOpenSendModal: () => void;
}

export const Header: React.FC<HeaderProps> = ({ onOpenSendModal }) => {
  const navigate = useNavigate();
  const location = useLocation();
  const dispatch = useAppDispatch();
  const { user, currentTenant, tenants } = useAppSelector((state) => state.auth);
  const [tenantMenuOpen, setTenantMenuOpen] = useState(false);
  const [userMenuOpen, setUserMenuOpen] = useState(false);

  const getPageTitle = (pathname: string) => {
    if (pathname === '/') return 'Overview & Telemetry';
    if (pathname.startsWith('/events')) return 'Live Events Stream';
    if (pathname.startsWith('/deliveries')) return 'Deliveries & Traces';
    if (pathname.startsWith('/dlq')) return 'Dead Letter Queue (DLQ)';
    if (pathname.startsWith('/sources')) return 'Inbound Sources';
    if (pathname.startsWith('/destinations')) return 'Destinations & Circuit Breakers';
    if (pathname.startsWith('/subscriptions')) return 'Subscriptions & Routing Rules';
    if (pathname.startsWith('/transformations')) return 'Transformation Studio';
    if (pathname.startsWith('/docs')) return 'Developer Documentation';
    if (pathname.startsWith('/apikeys')) return 'API Keys & Access';
    return 'Dashboard';
  };

  return (
    <header className="h-16 border-b border-zinc-800 bg-[#0c0c0e]/90 backdrop-blur-md px-6 flex items-center justify-between sticky top-0 z-30">
      {/* Left: Breadcrumbs / Title */}
      <div className="flex items-center space-x-4">
        {/* Tenant Switcher */}
        <div className="relative">
          <button
            onClick={() => setTenantMenuOpen(!tenantMenuOpen)}
            className="flex items-center space-x-2 px-3 py-1.5 rounded-lg bg-zinc-900 border border-zinc-800 hover:border-zinc-700 text-xs font-medium text-zinc-200 transition-colors"
          >
            <Building className="w-3.5 h-3.5 text-zinc-400" />
            <span className="max-w-[140px] truncate">
              {currentTenant ? currentTenant.name : 'Production Tenant'}
            </span>
            <ChevronDown className="w-3.5 h-3.5 text-zinc-500" />
          </button>

          {tenantMenuOpen && (
            <div className="absolute left-0 mt-2 w-56 bg-[#121215] border border-zinc-800 rounded-xl shadow-xl p-1.5 z-40 animate-in fade-in zoom-in-95 duration-100">
              <div className="px-2 py-1.5 text-[10px] font-mono text-zinc-500 uppercase">
                Active Tenant / Workspace
              </div>
              <div className="space-y-0.5 max-h-48 overflow-y-auto">
                {tenants.length === 0 ? (
                  <div className="px-2 py-1 text-xs text-zinc-400">Default Tenant (active)</div>
                ) : (
                  tenants.map((t) => (
                    <button
                      key={t.id}
                      onClick={() => {
                        dispatch(switchTenant(t));
                        setTenantMenuOpen(false);
                      }}
                      className="w-full flex items-center justify-between px-2 py-1.5 text-xs text-zinc-300 hover:text-white hover:bg-zinc-800/80 rounded-md transition-colors text-left"
                    >
                      <span className="truncate">{t.name}</span>
                      {currentTenant?.id === t.id && (
                        <Check className="w-3.5 h-3.5 text-emerald-400" />
                      )}
                    </button>
                  ))
                )}
              </div>
            </div>
          )}
        </div>

        <div className="h-4 w-px bg-zinc-800" />

        <div className="flex items-center space-x-2">
          <span className="text-sm font-semibold text-zinc-100">{getPageTitle(location.pathname)}</span>
        </div>
      </div>

      {/* Right Action buttons */}
      <div className="flex items-center space-x-3">
        {/* Quick Send Webhook Trigger */}
        <button
          onClick={onOpenSendModal}
          className="flex items-center space-x-1.5 text-xs font-medium px-3 py-1.5 rounded-lg bg-zinc-100 text-zinc-900 hover:bg-white transition-all shadow-sm active:scale-95"
        >
          <Send className="w-3.5 h-3.5" />
          <span>Send Test Webhook</span>
        </button>

        {/* Documentation Portal Switch */}
        <button
          onClick={() => navigate('/docs')}
          className={`flex items-center space-x-1.5 text-xs font-medium px-3 py-1.5 rounded-lg border transition-colors ${
            location.pathname.startsWith('/docs')
              ? 'bg-zinc-800 border-zinc-700 text-white'
              : 'bg-zinc-900/60 border-zinc-800 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900'
          }`}
        >
          <BookOpen className="w-3.5 h-3.5" />
          <span>Docs</span>
        </button>

        {/* User / Profile menu */}
        <div className="relative">
          <button
            onClick={() => setUserMenuOpen(!userMenuOpen)}
            className="flex items-center space-x-2 p-1.5 rounded-lg hover:bg-zinc-900 text-zinc-300 transition-colors"
          >
            <div className="w-7 h-7 rounded-full bg-zinc-800 border border-zinc-700 flex items-center justify-center text-xs font-semibold text-white">
              {user ? user.email.charAt(0).toUpperCase() : 'W'}
            </div>
          </button>

          {userMenuOpen && (
            <div className="absolute right-0 mt-2 w-52 bg-[#121215] border border-zinc-800 rounded-xl shadow-xl p-1.5 z-40 animate-in fade-in zoom-in-95 duration-100">
              <div className="px-3 py-2 border-b border-zinc-800">
                <div className="text-xs font-medium text-white truncate">
                  {user ? user.email : 'Admin Developer'}
                </div>
                <div className="text-[10px] text-zinc-500 font-mono">
                  {user?.is_admin ? 'Platform Admin' : 'Tenant Admin'}
                </div>
              </div>
              <div className="py-1">
                <button
                  onClick={() => {
                    navigate('/apikeys');
                    setUserMenuOpen(false);
                  }}
                  className="w-full flex items-center space-x-2 px-2.5 py-1.5 text-xs text-zinc-300 hover:text-white hover:bg-zinc-800/80 rounded-md transition-colors text-left"
                >
                  <UserIcon className="w-3.5 h-3.5 text-zinc-400" />
                  <span>API Keys & Credentials</span>
                </button>
                <button
                  onClick={() => {
                    dispatch(logout());
                    setUserMenuOpen(false);
                  }}
                  className="w-full flex items-center space-x-2 px-2.5 py-1.5 text-xs text-rose-400 hover:text-rose-300 hover:bg-rose-950/30 rounded-md transition-colors text-left"
                >
                  <LogOut className="w-3.5 h-3.5" />
                  <span>Log out / Reset</span>
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </header>
  );
};
