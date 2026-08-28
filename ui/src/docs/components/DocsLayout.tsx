import React, { useEffect, useState } from 'react';
import { useLocation } from 'react-router-dom';
import { DocsHeader } from './DocsHeader';
import { DocsSidebar } from './DocsSidebar';
import { DocsSearchModal } from './DocsSearchModal';
import { DocsTableOfContents } from './DocsTableOfContents';
import { DocHeading } from '../types';

interface DocsLayoutProps {
  children: React.ReactNode;
  headings?: DocHeading[];
}

export const DocsLayout: React.FC<DocsLayoutProps> = ({ children, headings = [] }) => {
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const location = useLocation();

  // Scroll to top or to hash anchor when route changes
  useEffect(() => {
    if (location.hash) {
      const id = location.hash.replace(/^#/, '');
      const element = document.getElementById(id);
      if (element) {
        setTimeout(() => {
          element.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }, 100);
        return;
      }
    }
    window.scrollTo(0, 0);
  }, [location.pathname, location.hash]);

  return (
    <div className="min-h-screen bg-[#09090b] text-zinc-100 flex flex-col font-sans selection:bg-emerald-500/20 selection:text-emerald-300">
      {/* Fixed Docs Header */}
      <DocsHeader
        onOpenSearch={() => setIsSearchOpen(true)}
        onToggleMobileMenu={() => setIsMobileMenuOpen((prev) => !prev)}
      />

      {/* Main 3-Column Workspace */}
      <div className="flex-1 flex max-w-7xl w-full mx-auto">
        {/* Left Sidebar */}
        <DocsSidebar
          isMobileOpen={isMobileMenuOpen}
          onCloseMobile={() => setIsMobileMenuOpen(false)}
        />

        {/* Center Content Reading Container */}
        <main className="flex-1 min-w-0 px-4 sm:px-8 md:px-12 py-8 lg:py-10 max-w-4xl mx-auto">
          {children}
        </main>

        {/* Right Table of Contents */}
        {headings && headings.length > 0 && (
          <DocsTableOfContents headings={headings} />
        )}
      </div>

      {/* Global Cmd+K Search Modal */}
      <DocsSearchModal
        isOpen={isSearchOpen}
        onClose={() => setIsSearchOpen(false)}
      />
    </div>
  );
};
