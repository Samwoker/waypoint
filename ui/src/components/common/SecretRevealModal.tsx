import React, { useState } from 'react';
import { Key, Copy, Check, AlertTriangle, ShieldAlert } from 'lucide-react';

interface SecretRevealModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  subtitle?: string;
  secret: string;
  warning?: string;
}

export const SecretRevealModal: React.FC<SecretRevealModalProps> = ({
  isOpen,
  onClose,
  title,
  subtitle = 'Store this credential in your environment variables or key vault.',
  secret,
  warning = 'This secret will NOT be shown again. Once you close this modal, it cannot be retrieved.',
}) => {
  const [copied, setCopied] = useState(false);

  if (!isOpen) return null;

  const handleCopy = () => {
    navigator.clipboard.writeText(secret);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-in fade-in duration-150">
      <div className="bg-[#121215] border border-zinc-800 rounded-3xl max-w-lg w-full p-6 shadow-2xl space-y-6 animate-in zoom-in-95 duration-150">
        <div className="flex items-start space-x-3.5">
          <div className="p-3 rounded-2xl bg-amber-500/10 text-amber-400 border border-amber-500/20 shrink-0">
            <Key className="w-6 h-6" />
          </div>
          <div className="space-y-1">
            <h3 className="text-lg font-bold text-white tracking-tight">{title}</h3>
            <p className="text-xs text-zinc-400 leading-relaxed">{subtitle}</p>
          </div>
        </div>

        {/* Warning Banner */}
        <div className="p-3.5 rounded-xl bg-amber-950/30 border border-amber-800/40 text-amber-300 text-xs flex items-start space-x-2.5">
          <AlertTriangle className="w-4 h-4 shrink-0 text-amber-400 mt-0.5" />
          <span className="leading-relaxed font-medium">{warning}</span>
        </div>

        {/* Secret Value Block */}
        <div className="space-y-1.5">
          <label className="block text-[11px] font-mono font-semibold uppercase tracking-wider text-zinc-400">
            Generated Secret
          </label>
          <div className="flex items-center space-x-2 bg-zinc-950 p-2.5 rounded-xl border border-zinc-800">
            <code className="flex-1 font-mono text-xs text-emerald-400 break-all select-all font-semibold px-2 py-1">
              {secret}
            </code>
            <button
              type="button"
              onClick={handleCopy}
              className="px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-white text-xs font-semibold flex items-center space-x-1.5 transition-colors shrink-0"
            >
              {copied ? (
                <>
                  <Check className="w-3.5 h-3.5 text-emerald-400" />
                  <span className="text-emerald-400">Copied!</span>
                </>
              ) : (
                <>
                  <Copy className="w-3.5 h-3.5" />
                  <span>Copy</span>
                </>
              )}
            </button>
          </div>
        </div>

        {/* Dismiss Button */}
        <div className="pt-2 border-t border-zinc-800/80 flex justify-end">
          <button
            type="button"
            onClick={onClose}
            className="px-5 py-2.5 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs transition-all shadow-md active:scale-95"
          >
            I Have Saved This Secret
          </button>
        </div>
      </div>
    </div>
  );
};
