import React, { useEffect, useState } from 'react';
import { AlignLeft } from 'lucide-react';
import { DocHeading } from '../types';

interface DocsTableOfContentsProps {
  headings: DocHeading[];
}

export const DocsTableOfContents: React.FC<DocsTableOfContentsProps> = ({ headings }) => {
  const [activeId, setActiveId] = useState<string>('');

  useEffect(() => {
    if (headings.length === 0) return;

    const handleScroll = () => {
      const headingElements = headings
        .map((h) => document.getElementById(h.id))
        .filter((el): el is HTMLElement => el !== null);

      const scrollPosition = window.scrollY + 120;

      for (let i = headingElements.length - 1; i >= 0; i--) {
        const el = headingElements[i];
        if (el.offsetTop <= scrollPosition) {
          setActiveId(el.id);
          return;
        }
      }

      if (headingElements.length > 0) {
        setActiveId(headingElements[0].id);
      }
    };

    window.addEventListener('scroll', handleScroll, { passive: true });
    handleScroll();

    return () => window.removeEventListener('scroll', handleScroll);
  }, [headings]);

  if (headings.length === 0) {
    return null;
  }

  return (
    <aside className="hidden xl:block w-60 shrink-0 sticky top-20 max-h-[calc(100vh-80px)] overflow-y-auto p-4 space-y-3">
      <div className="flex items-center space-x-2 text-[10px] font-mono font-bold uppercase tracking-wider text-zinc-500">
        <AlignLeft className="w-3.5 h-3.5 text-zinc-400" />
        <span>On This Page</span>
      </div>

      <nav className="space-y-1 text-xs border-l border-zinc-800/80 pl-2">
        {headings.map((h) => {
          const isActive = activeId === h.id;

          return (
            <a
              key={h.id}
              href={`#${h.id}`}
              className={`block py-1 transition-all ${
                h.level === 3 ? 'pl-3 text-[11px]' : 'font-medium'
              } ${
                isActive
                  ? 'text-emerald-400 font-semibold translate-x-0.5'
                  : 'text-zinc-400 hover:text-zinc-200'
              }`}
            >
              {h.text}
            </a>
          );
        })}
      </nav>
    </aside>
  );
};
