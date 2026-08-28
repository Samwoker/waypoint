import React, { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  FileText,
  Search,
  X,
  CornerDownLeft,
  ArrowUpDown,
} from 'lucide-react';
import { searchDocs } from '../docsContent';
import { SearchResultItem } from '../types';

interface DocsSearchModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const DocsSearchModal: React.FC<DocsSearchModalProps> = ({ isOpen, onClose }) => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResultItem[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 50);
      setQuery('');
      setResults([]);
      setSelectedIndex(0);
    }
  }, [isOpen]);

  // Handle global shortcut Cmd+K / Ctrl+K
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        if (isOpen) {
          onClose();
        } else {
          // Open handled by parent, or toggle
        }
      }
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  // Execute search
  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      setSelectedIndex(0);
      return;
    }
    const searchRes = searchDocs(query);
    setResults(searchRes);
    setSelectedIndex(0);
  }, [query]);

  const handleSelectResult = (item: SearchResultItem) => {
    navigate(item.item.route);
    onClose();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => (results.length > 0 ? (prev + 1) % results.length : 0));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => (results.length > 0 ? (prev - 1 + results.length) % results.length : 0));
    } else if (e.key === 'Enter' && results[selectedIndex]) {
      e.preventDefault();
      handleSelectResult(results[selectedIndex]);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-16 sm:pt-24 px-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-150">
      <div
        className="w-full max-w-2xl bg-[#0e0e11] border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col animate-in zoom-in-95 duration-150"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input Header */}
        <div className="flex items-center px-4 py-3.5 border-b border-zinc-800 bg-zinc-950/80">
          <Search className="w-4 h-4 text-zinc-400 mr-3 shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search docs, APIs, concepts, retries, headers, express..."
            className="w-full bg-transparent text-sm text-white placeholder-zinc-500 focus:outline-none font-sans"
          />
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded-lg text-zinc-500 hover:text-white hover:bg-zinc-800 transition-colors ml-2"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Results List */}
        <div className="max-h-[60vh] overflow-y-auto p-2 space-y-1 divide-y divide-zinc-900">
          {query.trim().length === 0 ? (
            <div className="py-12 text-center text-xs text-zinc-500 space-y-2">
              <Search className="w-8 h-8 text-zinc-600 mx-auto opacity-40" />
              <p>Type to search across RelayCore documentation, APIs, and guides</p>
              <div className="flex justify-center space-x-2 pt-2 text-[11px] font-mono text-zinc-600">
                <span className="px-2 py-0.5 rounded bg-zinc-900 border border-zinc-800">Quickstart</span>
                <span className="px-2 py-0.5 rounded bg-zinc-900 border border-zinc-800">HMAC Signatures</span>
                <span className="px-2 py-0.5 rounded bg-zinc-900 border border-zinc-800">Circuit Breakers</span>
              </div>
            </div>
          ) : results.length === 0 ? (
            <div className="py-12 text-center text-xs text-zinc-500 space-y-1">
              <p>No results found for &ldquo;<span className="text-zinc-300 font-semibold">{query}</span>&rdquo;</p>
              <p className="text-[11px]">Try searching for endpoints like &ldquo;POST /hooks&rdquo; or &ldquo;retry&rdquo;</p>
            </div>
          ) : (
            results.map((res, idx) => {
              const isSelected = selectedIndex === idx;

              return (
                <div
                  key={res.item.slug}
                  onClick={() => handleSelectResult(res)}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  className={`p-3 rounded-xl cursor-pointer transition-all flex items-start space-x-3 ${
                    isSelected
                      ? 'bg-zinc-800/90 text-white border border-zinc-700/60 shadow-sm'
                      : 'text-zinc-300 hover:bg-zinc-900/60'
                  }`}
                >
                  <div className="p-2 rounded-lg bg-zinc-900 border border-zinc-800 text-emerald-400 mt-0.5 shrink-0">
                    <FileText className="w-4 h-4" />
                  </div>
                  <div className="flex-1 min-w-0 space-y-1">
                    <div className="flex items-center justify-between">
                      <h4 className="text-xs font-bold text-white truncate">
                        {res.item.title}
                      </h4>
                      <span className="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-zinc-950 border border-zinc-800 text-zinc-400 shrink-0">
                        {res.item.category}
                      </span>
                    </div>

                    {res.headingMatch && (
                      <div className="text-[11px] text-emerald-400/90 font-medium">
                        Section: {res.headingMatch}
                      </div>
                    )}

                    <p className="text-[11px] text-zinc-400 line-clamp-2 leading-relaxed">
                      {res.snippet}
                    </p>
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Footer shortcuts */}
        <div className="px-4 py-2.5 border-t border-zinc-800 bg-zinc-950 flex items-center justify-between text-[10px] font-mono text-zinc-500">
          <div className="flex items-center space-x-4">
            <span className="flex items-center space-x-1">
              <ArrowUpDown className="w-3 h-3" />
              <span>Navigate</span>
            </span>
            <span className="flex items-center space-x-1">
              <CornerDownLeft className="w-3 h-3" />
              <span>Select</span>
            </span>
          </div>
          <span>ESC to close</span>
        </div>
      </div>
    </div>
  );
};
