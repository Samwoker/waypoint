import React from 'react';
import { useLocation, Link } from 'react-router-dom';
import { AlertCircle, ArrowRight, BookOpen, Home, Search } from 'lucide-react';
import { DocsLayout } from '../components/DocsLayout';
import { DocsBreadcrumbs } from '../components/DocsBreadcrumbs';
import { DocsPagination } from '../components/DocsPagination';
import { MarkdownRenderer } from '../components/MarkdownRenderer';
import { getDocBySlug, getPrevNextDocs } from '../docsContent';

export const DocDetailPage: React.FC = () => {
  const location = useLocation();
  const rawPath = location.pathname.replace(/^\/docs\/?/, '');
  const doc = getDocBySlug(rawPath);

  if (!doc) {
    return (
      <DocsLayout>
        <div className="py-16 text-center space-y-6 max-w-md mx-auto animate-in fade-in">
          <div className="w-12 h-12 rounded-2xl bg-amber-500/10 border border-amber-500/20 text-amber-400 flex items-center justify-center mx-auto">
            <AlertCircle className="w-6 h-6" />
          </div>

          <div className="space-y-2">
            <h1 className="text-2xl font-extrabold text-white tracking-tight">
              Documentation Page Not Found
            </h1>
            <p className="text-xs text-zinc-400 leading-relaxed">
              We couldn&apos;t find a document matching <code className="text-amber-400 bg-zinc-900 px-1.5 py-0.5 rounded border border-zinc-800 font-mono text-[11px]">{location.pathname}</code>.
            </p>
          </div>

          <div className="p-4 rounded-2xl bg-[#0e0e11] border border-zinc-800 text-left space-y-2.5">
            <span className="text-[10px] font-mono uppercase text-zinc-500 font-bold block">
              Suggested Entry Points:
            </span>
            <ul className="space-y-1.5 text-xs text-zinc-300">
              <li>
                <Link to="/docs/getting-started/quickstart" className="text-emerald-400 hover:underline flex items-center space-x-1.5">
                  <ArrowRight className="w-3 h-3" />
                  <span>5-Minute Quickstart Guide</span>
                </Link>
              </li>
              <li>
                <Link to="/docs/api/overview" className="text-emerald-400 hover:underline flex items-center space-x-1.5">
                  <ArrowRight className="w-3 h-3" />
                  <span>REST API Reference</span>
                </Link>
              </li>
              <li>
                <Link to="/docs/concepts/core-concepts" className="text-emerald-400 hover:underline flex items-center space-x-1.5">
                  <ArrowRight className="w-3 h-3" />
                  <span>Core Concepts & Pipeline</span>
                </Link>
              </li>
              <li>
                <Link to="/docs/integrations/expressjs" className="text-emerald-400 hover:underline flex items-center space-x-1.5">
                  <ArrowRight className="w-3 h-3" />
                  <span>Express.js Webhook Receiver</span>
                </Link>
              </li>
            </ul>
          </div>

          <Link
            to="/docs"
            className="inline-flex items-center space-x-2 px-4 py-2 rounded-xl text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 text-white transition-colors"
          >
            <Home className="w-3.5 h-3.5" />
            <span>Go to Documentation Home</span>
          </Link>
        </div>
      </DocsLayout>
    );
  }

  const { prev, next } = getPrevNextDocs(doc.slug);

  return (
    <DocsLayout headings={doc.headings}>
      <article className="animate-in fade-in duration-200">
        {/* Breadcrumb Navigation */}
        <DocsBreadcrumbs category={doc.category} title={doc.title} />

        {/* Dynamic Markdown Article Content */}
        <MarkdownRenderer content={doc.rawContent} currentSlug={doc.slug} />

        {/* Previous / Next Article Navigation Footer */}
        <DocsPagination prev={prev} next={next} />
      </article>
    </DocsLayout>
  );
};
