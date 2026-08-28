import React from 'react';
import { Link } from 'react-router-dom';
import { ChevronRight, Home } from 'lucide-react';

interface DocsBreadcrumbsProps {
  category: string;
  title: string;
}

export const DocsBreadcrumbs: React.FC<DocsBreadcrumbsProps> = ({ category, title }) => {
  const formattedCategory = category
    .split('-')
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ');

  return (
    <nav className="flex items-center space-x-1.5 text-xs text-zinc-500 mb-6 font-mono">
      <Link to="/docs" className="hover:text-zinc-300 flex items-center space-x-1 transition-colors">
        <Home className="w-3.5 h-3.5" />
        <span>Docs</span>
      </Link>
      <ChevronRight className="w-3.5 h-3.5 text-zinc-700" />
      <span className="text-zinc-400 capitalize">{formattedCategory}</span>
      <ChevronRight className="w-3.5 h-3.5 text-zinc-700" />
      <span className="text-emerald-400 font-semibold truncate max-w-xs">{title}</span>
    </nav>
  );
};
