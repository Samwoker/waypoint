import React from 'react';
import { Link } from 'react-router-dom';
import {
  ArrowRight,
  BookOpen,
  Command,
  LayoutDashboard,
  Menu,
  Search,
  Sparkles,
  Zap,
} from 'lucide-react';
import { useAppSelector } from '../../store/hooks';

interface DocsHeaderProps {
  onOpenSearch: () => void;
  onToggleMobileMenu: () => void;
}

export const DocsHeader: React.FC<DocsHeaderProps> = ({
  onOpenSearch,
  onToggleMobileMenu,
}) => {
  const { user, token } = useAppSelector((state) => state.auth);
  const isAuthenticated = !!(user && token);

  return (
    <header className="sticky top-0 z-30 h-16 border-b border-zinc-800 bg-[#09090b]/90 backdrop-blur-md px-4 sm:px-6 flex items-center justify-between">
      {/* Left: Brand & Docs Badge */}
      <div className="flex items-center space-x-3">
        {/* Mobile menu trigger */}
        <button
          type="button"
          onClick={onToggleMobileMenu}
          aria-label="Toggle navigation menu"
          className="lg:hidden p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
        >
          <Menu className="w-5 h-5" />
        </button>

        <Link to="/" className="flex items-center space-x-2.5 group">
          <div className="w-8 h-8 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center font-bold font-mono group-hover:scale-105 transition-transform">
            <Zap className="w-4 h-4" />
          </div>
          <div className="flex items-center space-x-2">
            <span className="font-bold text-white text-base tracking-tight">RelayCore</span>
            <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-semibold bg-zinc-800 text-emerald-400 border border-zinc-700/60">
              Docs
            </span>
          </div>
        </Link>
      </div>

      {/* Center: Search Button (Cmd+K) */}
      <div className="flex-1 max-w-md mx-4 hidden sm:block">
        <button
          type="button"
          onClick={onOpenSearch}
          className="w-full flex items-center justify-between px-3.5 py-1.5 rounded-xl bg-zinc-900/90 hover:bg-zinc-800/90 border border-zinc-800 hover:border-zinc-700 text-zinc-400 hover:text-zinc-200 transition-all text-xs shadow-inner"
        >
          <div className="flex items-center space-x-2">
            <Search className="w-3.5 h-3.5 text-zinc-500" />
            <span>Search documentation...</span>
          </div>
          <kbd className="flex items-center space-x-1 px-1.5 py-0.5 rounded bg-zinc-950 border border-zinc-800 text-[10px] font-mono text-zinc-500">
            <Command className="w-3 h-3" />
            <span>K</span>
          </kbd>
        </button>
      </div>

      {/* Right: Actions & Links */}
      <div className="flex items-center space-x-2">
        <button
          type="button"
          onClick={onOpenSearch}
          aria-label="Search"
          className="sm:hidden p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
        >
          <Search className="w-5 h-5" />
        </button>

        <Link
          to="/pricing"
          className="hidden md:inline-block px-3 py-1.5 rounded-lg text-xs font-medium text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
        >
          Pricing
        </Link>

        <a
          href="https://github.com/Samwoker/waypoint"
          target="_blank"
          rel="noopener noreferrer"
          className="p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors flex items-center space-x-1.5 text-xs font-medium"
          title="GitHub Repository"
        >
          <svg className="w-4 h-4 fill-current" viewBox="0 0 24 24">
            <path
              fillRule="evenodd"
              clipRule="evenodd"
              d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
            />
          </svg>
          <span className="hidden md:inline">GitHub</span>
        </a>

        {isAuthenticated ? (
          <Link
            to="/dashboard"
            className="flex items-center space-x-1.5 px-3.5 py-1.5 rounded-xl text-xs font-semibold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-md shadow-emerald-500/10 transition-all font-sans"
          >
            <LayoutDashboard className="w-3.5 h-3.5" />
            <span>Dashboard</span>
          </Link>
        ) : (
          <div className="flex items-center space-x-2">
            <Link
              to="/login"
              className="px-3 py-1.5 rounded-xl text-xs font-medium text-zinc-300 hover:text-white hover:bg-zinc-800 transition-colors"
            >
              Sign In
            </Link>
            <Link
              to="/signup"
              className="hidden sm:flex items-center space-x-1 px-3 py-1.5 rounded-xl text-xs font-semibold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-md transition-all font-sans"
            >
              <span>Start Free</span>
              <ArrowRight className="w-3 h-3" />
            </Link>
          </div>
        )}
      </div>
    </header>
  );
};
