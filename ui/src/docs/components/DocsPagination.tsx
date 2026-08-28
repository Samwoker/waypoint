import React from 'react';
import { Link } from 'react-router-dom';
import { ArrowLeft, ArrowRight } from 'lucide-react';
import { DocItem } from '../types';

interface DocsPaginationProps {
  prev?: DocItem;
  next?: DocItem;
}

export const DocsPagination: React.FC<DocsPaginationProps> = ({ prev, next }) => {
  if (!prev && !next) return null;

  return (
    <div className="mt-14 pt-8 border-t border-zinc-800/80 grid grid-cols-1 sm:grid-cols-2 gap-4">
      {prev ? (
        <Link
          to={prev.route}
          className="group p-4 rounded-2xl bg-[#0e0e11] hover:bg-zinc-900 border border-zinc-800 hover:border-zinc-700 transition-all flex flex-col items-start space-y-1 shadow-sm"
        >
          <div className="flex items-center space-x-1 text-[11px] font-mono text-zinc-500 group-hover:text-emerald-400 transition-colors">
            <ArrowLeft className="w-3.5 h-3.5 group-hover:-translate-x-1 transition-transform" />
            <span>PREVIOUS</span>
          </div>
          <div className="text-sm font-semibold text-zinc-200 group-hover:text-white transition-colors">
            {prev.title}
          </div>
        </Link>
      ) : (
        <div className="hidden sm:block" />
      )}

      {next ? (
        <Link
          to={next.route}
          className="group p-4 rounded-2xl bg-[#0e0e11] hover:bg-zinc-900 border border-zinc-800 hover:border-zinc-700 transition-all flex flex-col items-end space-y-1 sm:text-right shadow-sm"
        >
          <div className="flex items-center space-x-1 text-[11px] font-mono text-zinc-500 group-hover:text-emerald-400 transition-colors">
            <span>NEXT</span>
            <ArrowRight className="w-3.5 h-3.5 group-hover:translate-x-1 transition-transform" />
          </div>
          <div className="text-sm font-semibold text-zinc-200 group-hover:text-white transition-colors">
            {next.title}
          </div>
        </Link>
      ) : (
        <div className="hidden sm:block" />
      )}
    </div>
  );
};
