import React from 'react';
import { Link } from 'react-router-dom';
import { Zap, ShieldCheck, Heart } from 'lucide-react';

export const PublicFooter: React.FC = () => {
  return (
    <footer className="border-t border-zinc-800 bg-[#070709] text-zinc-400 text-xs">
      <div className="max-w-7xl mx-auto px-4 sm:px-8 py-12 lg:py-16 grid grid-cols-2 md:grid-cols-5 gap-8">
        {/* Col 1: Brand */}
        <div className="col-span-2 space-y-4">
          <Link to="/" className="flex items-center space-x-2.5">
            <div className="w-7 h-7 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center font-mono">
              <Zap className="w-3.5 h-3.5" />
            </div>
            <span className="font-bold text-white text-base">RelayCore</span>
          </Link>
          <p className="text-zinc-500 text-xs leading-relaxed max-w-sm">
            High-performance, multi-tenant webhook ingestion, cryptographic validation, JSONPath transformation, and resilient fan-out relay platform.
          </p>
          <div className="flex items-center space-x-2 text-[11px] font-mono text-zinc-500">
            <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
            <span>Built in Rust & Tokio • 99.999% SLA Target</span>
          </div>
        </div>

        {/* Col 2: Product */}
        <div className="space-y-3">
          <h4 className="text-xs font-mono font-bold text-white uppercase tracking-wider">Product</h4>
          <ul className="space-y-2">
            <li>
              <Link to="/features" className="hover:text-white transition-colors">
                Features
              </Link>
            </li>
            <li>
              <Link to="/pricing" className="hover:text-white transition-colors">
                Pricing & Free Tier
              </Link>
            </li>
            <li>
              <Link to="/docs/introduction/architecture" className="hover:text-white transition-colors">
                Architecture
              </Link>
            </li>
            <li>
              <Link to="/docs/concepts/event-lifecycle" className="hover:text-white transition-colors">
                Event Lifecycle
              </Link>
            </li>
          </ul>
        </div>

        {/* Col 3: Documentation */}
        <div className="space-y-3">
          <h4 className="text-xs font-mono font-bold text-white uppercase tracking-wider">Documentation</h4>
          <ul className="space-y-2">
            <li>
              <Link to="/docs/getting-started/quickstart" className="hover:text-white transition-colors">
                5-Min Quickstart
              </Link>
            </li>
            <li>
              <Link to="/docs/api/overview" className="hover:text-white transition-colors">
                REST API Reference
              </Link>
            </li>
            <li>
              <Link to="/docs/integrations/expressjs" className="hover:text-white transition-colors">
                Express.js Receiver
              </Link>
            </li>
            <li>
              <Link to="/docs/integrations/nodejs" className="hover:text-white transition-colors">
                Node.js SDK Guide
              </Link>
            </li>
            <li>
              <Link to="/docs/security/production-checklist" className="hover:text-white transition-colors">
                Security Checklist
              </Link>
            </li>
          </ul>
        </div>

        {/* Col 4: Platform & Support */}
        <div className="space-y-3">
          <h4 className="text-xs font-mono font-bold text-white uppercase tracking-wider">Platform</h4>
          <ul className="space-y-2">
            <li>
              <a
                href="https://github.com/Samwoker/waypoint"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-white transition-colors"
              >
                GitHub Repository
              </a>
            </li>
            <li>
              <Link to="/login" className="hover:text-white transition-colors">
                Customer Sign In
              </Link>
            </li>
            <li>
              <Link to="/signup" className="hover:text-white transition-colors">
                Create Free Account
              </Link>
            </li>
            <li>
              <Link to="/docs/troubleshooting/common-issues" className="hover:text-white transition-colors">
                Troubleshooting & FAQ
              </Link>
            </li>
          </ul>
        </div>
      </div>

      {/* Bottom Bar */}
      <div className="border-t border-zinc-900 bg-black/40 py-6 px-4 sm:px-8 text-center text-zinc-600 text-[11px] flex flex-col sm:flex-row items-center justify-between max-w-7xl mx-auto">
        <span>© {new Date().getFullYear()} RelayCore (Waypoint). All rights reserved.</span>
        <span className="mt-2 sm:mt-0">Enterprise-grade Webhook PaaS</span>
      </div>
    </footer>
  );
};
