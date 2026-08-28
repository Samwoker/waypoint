import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import {
  ChevronDown,
  ChevronRight,
  BookOpen,
  X,
} from 'lucide-react';
import { docCategories } from '../docsContent';

interface DocsSidebarProps {
  isMobileOpen: boolean;
  onCloseMobile: () => void;
}

export const DocsSidebar: React.FC<DocsSidebarProps> = ({
  isMobileOpen,
  onCloseMobile,
}) => {
  const location = useLocation();
  const currentPath = location.pathname;

  // Track collapsed state per category
  const [collapsedCategories, setCollapsedCategories] = useState<Record<string, boolean>>({});

  const toggleCategory = (catId: string) => {
    setCollapsedCategories((prev) => ({
      ...prev,
      [catId]: !prev[catId],
    }));
  };

  const sidebarContent = (
    <nav className="p-4 space-y-6">
      {/* Home / Overview link */}
      <div>
        <Link
          to="/docs"
          onClick={onCloseMobile}
          className={`flex items-center space-x-2 px-3 py-2 rounded-xl text-xs font-semibold transition-all ${
            currentPath === '/docs' || currentPath === '/docs/' || currentPath === '/docs/readme'
              ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-bold shadow-sm'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60'
          }`}
        >
          <BookOpen className="w-3.5 h-3.5" />
          <span>Documentation Home</span>
        </Link>
      </div>

      {/* Categories tree */}
      {docCategories.map((group) => {
        const isCollapsed = !!collapsedCategories[group.id];

        return (
          <div key={group.id} className="space-y-1.5">
            <button
              type="button"
              onClick={() => toggleCategory(group.id)}
              className="w-full flex items-center justify-between px-2 py-1 text-[10px] font-mono font-bold text-zinc-500 hover:text-zinc-300 uppercase tracking-wider text-left transition-colors"
            >
              <span>{group.label}</span>
              {isCollapsed ? (
                <ChevronRight className="w-3 h-3 text-zinc-600" />
              ) : (
                <ChevronDown className="w-3 h-3 text-zinc-600" />
              )}
            </button>

            {!isCollapsed && (
              <div className="space-y-0.5 pl-1 border-l border-zinc-800/60 ml-2">
                {group.items.map((item) => {
                  const isActive =
                    currentPath === item.route ||
                    currentPath === `/docs/${item.slug}` ||
                    (item.slug === 'readme' && currentPath === '/docs');

                  return (
                    <Link
                      key={item.id}
                      to={item.route}
                      onClick={onCloseMobile}
                      className={`block px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${
                        isActive
                          ? 'bg-zinc-800 text-white font-semibold border-l-2 border-emerald-500 shadow-sm'
                          : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/50'
                      }`}
                    >
                      {item.title}
                    </Link>
                  );
                })}
              </div>
            )}
          </div>
        );
      })}
    </nav>
  );

  return (
    <>
      {/* Desktop Sidebar */}
      <aside className="hidden lg:block w-64 xl:w-72 shrink-0 border-r border-zinc-800 bg-[#09090b] overflow-y-auto h-[calc(100vh-64px)] sticky top-16 scrollbar-thin scrollbar-thumb-zinc-800">
        {sidebarContent}
      </aside>

      {/* Mobile Drawer */}
      {isMobileOpen && (
        <div className="fixed inset-0 z-50 lg:hidden">
          {/* Backdrop */}
          <div
            className="fixed inset-0 bg-black/70 backdrop-blur-sm transition-opacity"
            onClick={onCloseMobile}
          />

          {/* Drawer content */}
          <div className="fixed inset-y-0 left-0 w-72 bg-[#0c0c0e] border-r border-zinc-800 shadow-2xl flex flex-col z-10 animate-in slide-in-from-left duration-200">
            <div className="h-16 px-4 border-b border-zinc-800 flex items-center justify-between shrink-0">
              <div className="flex items-center space-x-2">
                <BookOpen className="w-4 h-4 text-emerald-400" />
                <span className="font-bold text-white text-sm">Navigation</span>
              </div>
              <button
                type="button"
                onClick={onCloseMobile}
                className="p-1.5 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="flex-1 overflow-y-auto">{sidebarContent}</div>
          </div>
        </div>
      )}
    </>
  );
};
