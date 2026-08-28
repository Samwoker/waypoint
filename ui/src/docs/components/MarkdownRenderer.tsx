import React, { useEffect, useRef, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import {
  Check,
  Copy,
  ExternalLink,
  Hash,
  AlertCircle,
  AlertTriangle,
  Info,
  Lightbulb,
  ShieldAlert,
} from 'lucide-react';
import Prism from 'prismjs';
import mermaid from 'mermaid';
import { slugify } from '../docsContent';

// Load Prism language grammars
import 'prismjs/components/prism-bash';
import 'prismjs/components/prism-json';
import 'prismjs/components/prism-typescript';
import 'prismjs/components/prism-javascript';
import 'prismjs/components/prism-python';
import 'prismjs/components/prism-rust';
import 'prismjs/components/prism-http';
import 'prismjs/components/prism-sql';
import 'prismjs/components/prism-yaml';
import 'prismjs/components/prism-docker';

// Initialize Mermaid for dark theme
mermaid.initialize({
  startOnLoad: false,
  theme: 'dark',
  themeVariables: {
    darkMode: true,
    background: '#09090b',
    primaryColor: '#10b981',
    primaryTextColor: '#f4f4f5',
    primaryBorderColor: '#27272a',
    lineColor: '#71717a',
    secondaryColor: '#18181b',
    tertiaryColor: '#121215',
  },
  securityLevel: 'loose',
});

interface MarkdownRendererProps {
  content: string;
  currentSlug: string;
}

// Code block with Copy button
const CodeSnippet: React.FC<{ code: string; language: string }> = ({ code, language }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(code.trim());
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const cleanLang = language.toLowerCase().trim() || 'plaintext';
  let highlighted = code;

  try {
    if (Prism.languages[cleanLang]) {
      highlighted = Prism.highlight(code, Prism.languages[cleanLang], cleanLang);
    }
  } catch (_) {
    // fallback to unhighlighted
  }

  return (
    <div className="my-5 rounded-2xl bg-[#0e0e11] border border-zinc-800/90 overflow-hidden shadow-lg group">
      {/* Code Header */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-zinc-800/80 bg-zinc-950/60">
        <span className="text-[11px] font-mono font-semibold uppercase text-zinc-400">
          {cleanLang}
        </span>
        <button
          type="button"
          onClick={handleCopy}
          aria-label="Copy code"
          className="flex items-center space-x-1.5 px-2.5 py-1 rounded-lg text-xs font-medium text-zinc-400 hover:text-white bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-800 transition-colors"
        >
          {copied ? (
            <>
              <Check className="w-3.5 h-3.5 text-emerald-400" />
              <span className="text-emerald-400 text-[11px]">Copied</span>
            </>
          ) : (
            <>
              <Copy className="w-3.5 h-3.5" />
              <span className="text-[11px]">Copy</span>
            </>
          )}
        </button>
      </div>

      {/* Code Body */}
      <div className="p-4 overflow-x-auto text-xs font-mono leading-relaxed text-zinc-200">
        <pre className="!bg-transparent !p-0 !m-0">
          <code
            className={`language-${cleanLang}`}
            dangerouslySetInnerHTML={{ __html: highlighted }}
          />
        </pre>
      </div>
    </div>
  );
};

// Interactive Mermaid Diagram Component
const MermaidDiagram: React.FC<{ chart: string; id: string }> = ({ chart, id }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [svg, setSvg] = useState<string>('');
  const [error, setError] = useState<boolean>(false);

  useEffect(() => {
    let isMounted = true;
    const renderChart = async () => {
      try {
        const uniqueId = `mermaid-${id}-${Math.random().toString(36).substr(2, 9)}`;
        const { svg: renderedSvg } = await mermaid.render(uniqueId, chart.trim());
        if (isMounted) {
          setSvg(renderedSvg);
          setError(false);
        }
      } catch (err) {
        if (isMounted) {
          console.warn('Mermaid render error:', err);
          setError(true);
        }
      }
    };

    renderChart();
    return () => {
      isMounted = false;
    };
  }, [chart, id]);

  if (error || !svg) {
    return <CodeSnippet code={chart} language="mermaid" />;
  }

  return (
    <div className="my-6 p-6 rounded-2xl bg-[#0e0e11] border border-zinc-800 flex justify-center overflow-x-auto shadow-md">
      <div
        ref={containerRef}
        className="mermaid-wrapper max-w-full"
        dangerouslySetInnerHTML={{ __html: svg }}
      />
    </div>
  );
};

export const MarkdownRenderer: React.FC<MarkdownRendererProps> = ({ content, currentSlug }) => {
  const navigate = useNavigate();

  // Helper to resolve relative markdown links (e.g. "./sources.md" -> "/docs/api/sources")
  const resolveDocLink = (href: string): string => {
    if (href.startsWith('http://') || href.startsWith('https://') || href.startsWith('mailto:')) {
      return href;
    }

    if (href.startsWith('#')) {
      return href;
    }

    // Convert relative markdown paths
    let target = href.replace(/\.md$/, '').replace(/\.md#/, '#');

    if (target.startsWith('./')) {
      const currentFolder = currentSlug.includes('/') ? currentSlug.split('/')[0] : '';
      target = target.replace(/^\.\//, currentFolder ? `${currentFolder}/` : '');
    } else if (target.startsWith('../')) {
      target = target.replace(/^\.\.\//, '');
    }

    return `/docs/${target.replace(/^\/+/, '')}`;
  };

  // Custom parser to split blocks: Headings, Code blocks, Mermaid, Callouts, Tables, Paragraphs
  const renderElements = () => {
    const lines = content.split('\n');
    const elements: React.ReactNode[] = [];
    let i = 0;

    while (i < lines.length) {
      const line = lines[i];
      const trimmed = line.trim();

      // 1. Code Block / Mermaid Block
      if (trimmed.startsWith('```')) {
        const lang = trimmed.slice(3).trim();
        const codeLines: string[] = [];
        i++;
        while (i < lines.length && !lines[i].trim().startsWith('```')) {
          codeLines.push(lines[i]);
          i++;
        }
        i++; // skip closing ```
        const codeContent = codeLines.join('\n');

        if (lang === 'mermaid') {
          elements.push(
            <MermaidDiagram
              key={`mermaid-${i}`}
              chart={codeContent}
              id={`m-${i}`}
            />
          );
        } else {
          elements.push(
            <CodeSnippet
              key={`code-${i}`}
              code={codeContent}
              language={lang}
            />
          );
        }
        continue;
      }

      // 2. Headings (H1, H2, H3, H4)
      if (trimmed.startsWith('# ') && i < 5) {
        // Main H1 Title
        const titleText = trimmed.replace(/^#\s+/, '').replace(/[#*`_]/g, '').trim();
        elements.push(
          <h1
            key={`h1-${i}`}
            className="text-3xl font-extrabold text-white tracking-tight pt-2 pb-1"
          >
            {titleText}
          </h1>
        );
        i++;
        continue;
      }

      if (trimmed.startsWith('## ')) {
        const hText = trimmed.replace(/^##\s+/, '').replace(/[#*`_]/g, '').trim();
        const hId = slugify(hText);
        elements.push(
          <h2
            key={`h2-${i}`}
            id={hId}
            className="group text-xl font-bold text-white tracking-tight mt-10 mb-4 pb-2 border-b border-zinc-800/80 flex items-center space-x-2 scroll-mt-24"
          >
            <span>{hText}</span>
            <a
              href={`#${hId}`}
              className="opacity-0 group-hover:opacity-100 text-zinc-500 hover:text-emerald-400 transition-opacity p-1"
              aria-label={`Link to ${hText}`}
            >
              <Hash className="w-4 h-4" />
            </a>
          </h2>
        );
        i++;
        continue;
      }

      if (trimmed.startsWith('### ')) {
        const hText = trimmed.replace(/^###\s+/, '').replace(/[#*`_]/g, '').trim();
        const hId = slugify(hText);
        elements.push(
          <h3
            key={`h3-${i}`}
            id={hId}
            className="group text-base font-semibold text-zinc-100 mt-6 mb-3 flex items-center space-x-2 scroll-mt-24"
          >
            <span>{hText}</span>
            <a
              href={`#${hId}`}
              className="opacity-0 group-hover:opacity-100 text-zinc-500 hover:text-emerald-400 transition-opacity p-1"
              aria-label={`Link to ${hText}`}
            >
              <Hash className="w-3.5 h-3.5" />
            </a>
          </h3>
        );
        i++;
        continue;
      }

      // 3. Blockquotes / Callouts
      if (trimmed.startsWith('>')) {
        const quoteLines: string[] = [];
        while (i < lines.length && lines[i].trim().startsWith('>')) {
          quoteLines.push(lines[i].replace(/^>\s?/, ''));
          i++;
        }
        const fullQuote = quoteLines.join(' ');
        const isWarning =
          fullQuote.includes('[!WARNING]') ||
          fullQuote.includes('[!DANGER]') ||
          fullQuote.toUpperCase().includes('WARNING:') ||
          fullQuote.toUpperCase().includes('CRITICAL');
        const isTip = fullQuote.includes('[!TIP]') || fullQuote.toUpperCase().includes('TIP:');
        const isImportant =
          fullQuote.includes('[!IMPORTANT]') || fullQuote.toUpperCase().includes('IMPORTANT:');

        const cleanText = fullQuote
          .replace(/\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION|DANGER)\]/g, '')
          .trim();

        elements.push(
          <div
            key={`callout-${i}`}
            className={`my-5 p-4 rounded-2xl border flex items-start space-x-3.5 text-xs leading-relaxed ${
              isWarning
                ? 'bg-rose-950/20 border-rose-800/40 text-rose-200'
                : isTip
                ? 'bg-emerald-950/20 border-emerald-800/40 text-emerald-200'
                : isImportant
                ? 'bg-amber-950/20 border-amber-800/40 text-amber-200'
                : 'bg-zinc-900/60 border-zinc-800 text-zinc-300'
            }`}
          >
            <div className="shrink-0 mt-0.5">
              {isWarning ? (
                <AlertTriangle className="w-4 h-4 text-rose-400" />
              ) : isTip ? (
                <Lightbulb className="w-4 h-4 text-emerald-400" />
              ) : isImportant ? (
                <AlertCircle className="w-4 h-4 text-amber-400" />
              ) : (
                <Info className="w-4 h-4 text-blue-400" />
              )}
            </div>
            <div className="flex-1 space-y-1">
              <span className="font-semibold block uppercase text-[10px] font-mono tracking-wider">
                {isWarning ? 'Warning' : isTip ? 'Tip' : isImportant ? 'Important' : 'Note'}
              </span>
              <p>{cleanText}</p>
            </div>
          </div>
        );
        continue;
      }

      // 4. Tables (Markdown tables)
      if (trimmed.startsWith('|') && trimmed.endsWith('|')) {
        const tableLines: string[] = [];
        while (i < lines.length && lines[i].trim().startsWith('|') && lines[i].trim().endsWith('|')) {
          tableLines.push(lines[i]);
          i++;
        }

        if (tableLines.length >= 2) {
          const headerRow = tableLines[0]
            .split('|')
            .slice(1, -1)
            .map((c) => c.trim());
          const bodyRows = tableLines.slice(2).map((row) =>
            row
              .split('|')
              .slice(1, -1)
              .map((c) => c.trim())
          );

          elements.push(
            <div key={`table-${i}`} className="my-6 overflow-x-auto rounded-2xl border border-zinc-800 bg-[#0e0e11] shadow-sm">
              <table className="w-full text-left text-xs border-collapse font-sans">
                <thead>
                  <tr className="border-b border-zinc-800 bg-zinc-950/80 font-mono text-[10px] uppercase text-zinc-400">
                    {headerRow.map((th, idx) => (
                      <th key={idx} className="py-3 px-4 font-semibold">
                        {th}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-800/60">
                  {bodyRows.map((row, rIdx) => (
                    <tr key={rIdx} className="hover:bg-zinc-900/40 transition-colors">
                      {row.map((cell, cIdx) => (
                        <td key={cIdx} className="py-3 px-4 text-zinc-300">
                          {renderInlineText(cell)}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          );
          continue;
        }
      }

      // 5. Horizontal Rule
      if (trimmed === '---' || trimmed === '***') {
        elements.push(<hr key={`hr-${i}`} className="my-8 border-zinc-800/80" />);
        i++;
        continue;
      }

      // 6. Lists (Unordered / Ordered)
      if (trimmed.startsWith('- ') || trimmed.startsWith('* ') || /^\d+\.\s/.test(trimmed)) {
        const listItems: string[] = [];
        const isOrdered = /^\d+\.\s/.test(trimmed);

        while (
          i < lines.length &&
          (lines[i].trim().startsWith('- ') ||
            lines[i].trim().startsWith('* ') ||
            /^\d+\.\s/.test(lines[i].trim()))
        ) {
          listItems.push(lines[i].trim().replace(/^[-*]\s+|\d+\.\s+/, ''));
          i++;
        }

        if (isOrdered) {
          elements.push(
            <ol key={`ol-${i}`} className="my-4 pl-6 space-y-1.5 text-xs text-zinc-300 list-decimal leading-relaxed">
              {listItems.map((item, idx) => (
                <li key={idx}>{renderInlineText(item)}</li>
              ))}
            </ol>
          );
        } else {
          elements.push(
            <ul key={`ul-${i}`} className="my-4 pl-5 space-y-1.5 text-xs text-zinc-300 list-disc leading-relaxed marker:text-emerald-500">
              {listItems.map((item, idx) => (
                <li key={idx}>{renderInlineText(item)}</li>
              ))}
            </ul>
          );
        }
        continue;
      }

      // 7. Regular Paragraphs
      if (trimmed.length > 0) {
        elements.push(
          <p key={`p-${i}`} className="my-3 text-xs leading-relaxed text-zinc-300 font-sans">
            {renderInlineText(trimmed)}
          </p>
        );
      }

      i++;
    }

    return elements;
  };

  // Helper to parse inline markdown: bold, code, links, method badges
  const renderInlineText = (text: string): React.ReactNode => {
    // Regex for inline code: `code`
    // Regex for links: [text](href)
    // Regex for bold: **text**
    const parts: React.ReactNode[] = [];
    let remaining = text;
    let keyIdx = 0;

    // First check for HTTP method pills at the beginning of endpoint strings (e.g. "POST /api/v1/...")
    const methodMatch = remaining.match(/^(GET|POST|PUT|PATCH|DELETE)\s+(\/[^\s]+)/);
    if (methodMatch) {
      const [full, method, endpoint] = methodMatch;
      const methodColors: { [k: string]: string } = {
        GET: 'bg-blue-500/10 text-blue-400 border-blue-500/20',
        POST: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
        PUT: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
        PATCH: 'bg-purple-500/10 text-purple-400 border-purple-500/20',
        DELETE: 'bg-rose-500/10 text-rose-400 border-rose-500/20',
      };

      parts.push(
        <span key={`method-${keyIdx++}`} className="inline-flex items-center space-x-2 my-1 font-mono font-bold">
          <span className={`px-2 py-0.5 rounded-md text-[10px] uppercase border ${methodColors[method] || 'bg-zinc-800 text-zinc-300'}`}>
            {method}
          </span>
          <code className="text-white text-xs">{endpoint}</code>
        </span>
      );
      remaining = remaining.slice(full.length);
    }

    // Tokenize markdown inline elements
    const inlineRegex = /(`[^`]+`)|(\[[^\]]+\]\([^)]+\))|(\*\*[^*]+\*\*)|(_[^_]+_)/g;
    let lastIndex = 0;
    let match;

    while ((match = inlineRegex.exec(remaining)) !== null) {
      // Add text before match
      if (match.index > lastIndex) {
        parts.push(remaining.substring(lastIndex, match.index));
      }

      const matchText = match[0];

      if (matchText.startsWith('`') && matchText.endsWith('`')) {
        // Inline code
        const code = matchText.slice(1, -1);
        parts.push(
          <code
            key={`inline-code-${keyIdx++}`}
            className="px-1.5 py-0.5 rounded-md bg-zinc-900 border border-zinc-800 text-emerald-400 font-mono text-[11px]"
          >
            {code}
          </code>
        );
      } else if (matchText.startsWith('[') && matchText.includes('](')) {
        // Link
        const linkText = matchText.match(/\[([^\]]+)\]/)?.[1] || '';
        const href = matchText.match(/\(([^)]+)\)/)?.[1] || '';
        const isExternal = href.startsWith('http://') || href.startsWith('https://');
        const resolvedHref = resolveDocLink(href);

        if (isExternal) {
          parts.push(
            <a
              key={`link-${keyIdx++}`}
              href={resolvedHref}
              target="_blank"
              rel="noopener noreferrer"
              className="text-emerald-400 hover:text-emerald-300 underline underline-offset-2 inline-flex items-center space-x-0.5 font-medium"
            >
              <span>{linkText}</span>
              <ExternalLink className="w-3 h-3 ml-0.5 inline-block opacity-70" />
            </a>
          );
        } else {
          parts.push(
            <Link
              key={`link-${keyIdx++}`}
              to={resolvedHref}
              className="text-emerald-400 hover:text-emerald-300 font-medium underline underline-offset-2 transition-colors"
            >
              {linkText}
            </Link>
          );
        }
      } else if (matchText.startsWith('**') && matchText.endsWith('**')) {
        // Bold
        parts.push(
          <strong key={`bold-${keyIdx++}`} className="font-bold text-white">
            {matchText.slice(2, -2)}
          </strong>
        );
      } else {
        parts.push(matchText);
      }

      lastIndex = inlineRegex.lastIndex;
    }

    if (lastIndex < remaining.length) {
      parts.push(remaining.substring(lastIndex));
    }

    return <>{parts}</>;
  };

  return <div className="space-y-3 prose prose-invert max-w-none">{renderElements()}</div>;
};
