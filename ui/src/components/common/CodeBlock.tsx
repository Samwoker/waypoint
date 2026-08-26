import React, { useEffect, useState } from 'react';
import { Check, Copy, Terminal } from 'lucide-react';
import Prism from 'prismjs';
import 'prismjs/components/prism-bash';
import 'prismjs/components/prism-javascript';
import 'prismjs/components/prism-typescript';
import 'prismjs/components/prism-python';
import 'prismjs/components/prism-rust';
import 'prismjs/components/prism-go';
import 'prismjs/components/prism-json';
import 'prismjs/components/prism-http';

export interface CodeSnippet {
  language: 'cURL' | 'TypeScript' | 'Python' | 'Rust' | 'Go' | 'JSON' | 'HTTP' | string;
  code: string;
}

interface CodeBlockProps {
  snippets?: CodeSnippet[];
  singleCode?: string;
  singleLang?: string;
  title?: string;
}

const getPrismLang = (lang: string): string => {
  const l = lang.toLowerCase();
  if (l === 'curl' || l === 'bash' || l === 'sh') return 'bash';
  if (l === 'typescript' || l === 'ts' || l === 'tsx') return 'typescript';
  if (l === 'javascript' || l === 'js') return 'javascript';
  if (l === 'python' || l === 'py') return 'python';
  if (l === 'rust' || l === 'rs') return 'rust';
  if (l === 'go' || l === 'golang') return 'go';
  if (l === 'json') return 'json';
  if (l === 'http') return 'http';
  return 'javascript';
};

export const CodeBlock: React.FC<CodeBlockProps> = ({
  snippets,
  singleCode,
  singleLang = 'bash',
  title,
}) => {
  const [activeTab, setActiveTab] = useState(0);
  const [copied, setCopied] = useState(false);

  const activeSnippet = snippets ? snippets[activeTab] : null;
  const currentCode = activeSnippet ? activeSnippet.code : singleCode || '';
  const currentLang = activeSnippet ? activeSnippet.language : singleLang;

  const handleCopy = () => {
    navigator.clipboard.writeText(currentCode);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getHighlightedHtml = (code: string, lang: string) => {
    const prismLang = getPrismLang(lang);
    const grammar = Prism.languages[prismLang] || Prism.languages.javascript;
    try {
      return Prism.highlight(code, grammar, prismLang);
    } catch {
      return code;
    }
  };

  return (
    <div className="rounded-xl border border-zinc-800/80 bg-[#0d0d10] overflow-hidden shadow-2xl transition-all hover:border-zinc-700/80 group my-4">
      {/* Header bar */}
      <div className="flex items-center justify-between px-4 py-2.5 bg-[#141418] border-b border-zinc-800/80">
        <div className="flex items-center space-x-3">
          <div className="flex items-center space-x-1.5">
            <div className="w-2.5 h-2.5 rounded-full bg-rose-500/80" />
            <div className="w-2.5 h-2.5 rounded-full bg-amber-500/80" />
            <div className="w-2.5 h-2.5 rounded-full bg-emerald-500/80" />
          </div>

          {title && (
            <span className="text-xs font-mono font-medium text-zinc-400 border-l border-zinc-800 pl-3">
              {title}
            </span>
          )}

          {snippets && snippets.length > 1 && (
            <div className="flex items-center space-x-1 bg-zinc-950/80 p-0.5 rounded-lg border border-zinc-800/60 ml-2">
              {snippets.map((snip, idx) => (
                <button
                  key={snip.language}
                  onClick={() => setActiveTab(idx)}
                  className={`px-2.5 py-0.5 text-[11px] font-mono rounded-md transition-all ${
                    activeTab === idx
                      ? 'bg-zinc-800 text-white font-semibold shadow-sm border border-zinc-700/50'
                      : 'text-zinc-400 hover:text-zinc-200'
                  }`}
                >
                  {snip.language}
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="flex items-center space-x-2">
          {!snippets && (
            <span className="text-[10px] font-mono uppercase px-2 py-0.5 rounded bg-zinc-900 text-zinc-400 border border-zinc-800">
              {singleLang}
            </span>
          )}
          <button
            onClick={handleCopy}
            className="flex items-center space-x-1.5 px-2.5 py-1 rounded-md bg-zinc-900/90 hover:bg-zinc-800 border border-zinc-800 text-zinc-300 hover:text-white transition-all text-xs font-mono shadow-sm active:scale-95"
            title="Copy code"
          >
            {copied ? (
              <>
                <Check className="w-3.5 h-3.5 text-emerald-400" />
                <span className="text-emerald-400 text-[11px]">Copied</span>
              </>
            ) : (
              <>
                <Copy className="w-3.5 h-3.5 text-zinc-400 group-hover:text-zinc-200" />
                <span className="text-[11px]">Copy</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Radiant Code Display Body */}
      <div className="p-4 overflow-x-auto font-mono text-xs leading-relaxed bg-[#0a0a0d]">
        <pre className="prism-highlight text-zinc-100 selection:bg-indigo-900/60 selection:text-white">
          <code
            dangerouslySetInnerHTML={{
              __html: getHighlightedHtml(currentCode, currentLang),
            }}
          />
        </pre>
      </div>
    </div>
  );
};
