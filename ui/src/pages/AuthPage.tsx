import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  AlertCircle,
  ArrowRight,
  Building,
  Key,
  Layers,
  Loader2,
  Lock,
  Mail,
  ShieldCheck,
  Sparkles,
  Zap,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  clearAuthError,
  loginRequest,
  registerRequest,
} from '../store/slices/authSlice';

export const AuthPage: React.FC = () => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { user, token, isLoading, error } = useAppSelector((state) => state.auth);

  const [mode, setMode] = useState<'signin' | 'register'>('signin');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [tenantName, setTenantName] = useState('');

  // If already logged in, redirect to dashboard
  useEffect(() => {
    if (user && token) {
      navigate('/');
    }
  }, [user, token, navigate]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (mode === 'signin') {
      dispatch(loginRequest({ email, pass: password }));
    } else {
      if (!tenantName.trim()) {
        alert('Please enter your Organization / Workspace name');
        return;
      }
      dispatch(registerRequest({ email, pass: password, tenantName }));
    }
  };

  const handleDemoLogin = (demoEmail: string, demoPass: string) => {
    setEmail(demoEmail);
    setPassword(demoPass);
    dispatch(loginRequest({ email: demoEmail, pass: demoPass }));
  };

  return (
    <div className="min-h-screen bg-[#09090b] flex flex-col justify-center items-center p-6 bg-grid-pattern relative overflow-hidden">
      {/* Background ambient lighting */}
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-96 h-96 bg-emerald-500/10 rounded-full blur-3xl pointer-events-none" />
      <div className="absolute bottom-1/4 right-1/4 w-80 h-80 bg-blue-500/10 rounded-full blur-3xl pointer-events-none" />

      {/* Main Authentication Card */}
      <div className="w-full max-w-md bg-[#121215]/90 border border-zinc-800 backdrop-blur-xl rounded-3xl p-8 shadow-2xl space-y-6 relative z-10 animate-in fade-in zoom-in-95 duration-200">
        {/* Brand Header */}
        <div className="text-center space-y-2">
          <div className="inline-flex p-3 rounded-2xl bg-gradient-to-tr from-zinc-800 to-zinc-700 border border-zinc-600 shadow-inner">
            <Layers className="w-6 h-6 text-white" />
          </div>
          <h1 className="text-2xl font-extrabold text-white tracking-tight">Waypoint Cloud</h1>
          <p className="text-xs text-zinc-400">
            Multi-Tenant Webhook Ingestion, Cryptographic Verification & Relay Engine
          </p>
        </div>

        {/* Tab Switcher: Sign In vs Register */}
        <div className="flex bg-zinc-950 p-1 rounded-xl border border-zinc-800/80">
          <button
            type="button"
            onClick={() => {
              setMode('signin');
              dispatch(clearAuthError());
            }}
            className={`flex-1 py-2 text-xs font-semibold rounded-lg transition-all ${
              mode === 'signin'
                ? 'bg-zinc-800 text-white shadow-sm border border-zinc-700/60'
                : 'text-zinc-400 hover:text-zinc-200'
            }`}
          >
            Sign In
          </button>
          <button
            type="button"
            onClick={() => {
              setMode('register');
              dispatch(clearAuthError());
            }}
            className={`flex-1 py-2 text-xs font-semibold rounded-lg transition-all ${
              mode === 'register'
                ? 'bg-zinc-800 text-white shadow-sm border border-zinc-700/60'
                : 'text-zinc-400 hover:text-zinc-200'
            }`}
          >
            Create Organization
          </button>
        </div>

        {/* Error Alert */}
        {error && (
          <div className="p-3 rounded-xl bg-rose-950/40 border border-rose-800/60 text-rose-300 text-xs flex items-center space-x-2 animate-in fade-in">
            <AlertCircle className="w-4 h-4 shrink-0 text-rose-400" />
            <span>{error}</span>
          </div>
        )}

        {/* Form */}
        <form onSubmit={handleSubmit} className="space-y-4">
          {mode === 'register' && (
            <div className="space-y-1.5">
              <label className="block text-xs font-mono text-zinc-400">Organization / Workspace Name</label>
              <div className="relative">
                <Building className="w-4 h-4 text-zinc-500 absolute left-3 top-2.5" />
                <input
                  type="text"
                  required
                  placeholder="Acme Global Corp"
                  value={tenantName}
                  onChange={(e) => setTenantName(e.target.value)}
                  className="w-full pl-9 pr-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-xl focus:outline-none focus:border-zinc-600 transition-colors"
                />
              </div>
            </div>
          )}

          <div className="space-y-1.5">
            <label className="block text-xs font-mono text-zinc-400">Work Email Address</label>
            <div className="relative">
              <Mail className="w-4 h-4 text-zinc-500 absolute left-3 top-2.5" />
              <input
                type="email"
                required
                placeholder="dev@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="w-full pl-9 pr-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-xl focus:outline-none focus:border-zinc-600 transition-colors"
              />
            </div>
          </div>

          <div className="space-y-1.5">
            <label className="block text-xs font-mono text-zinc-400">Password</label>
            <div className="relative">
              <Lock className="w-4 h-4 text-zinc-500 absolute left-3 top-2.5" />
              <input
                type="password"
                required
                placeholder="••••••••••••"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full pl-9 pr-3 py-2 text-xs font-mono text-zinc-100 bg-[#09090b] border border-zinc-800 rounded-xl focus:outline-none focus:border-zinc-600 transition-colors"
              />
            </div>
          </div>

          <button
            type="submit"
            disabled={isLoading}
            className="w-full py-2.5 px-4 rounded-xl bg-white text-zinc-950 hover:bg-zinc-200 font-semibold text-xs transition-all shadow-md active:scale-95 disabled:opacity-50 flex items-center justify-center space-x-2"
          >
            {isLoading ? (
              <Loader2 className="w-4 h-4 animate-spin text-zinc-950" />
            ) : (
              <>
                <span>{mode === 'signin' ? 'Sign In to Workspace' : 'Create Organization & Sign In'}</span>
                <ArrowRight className="w-4 h-4" />
              </>
            )}
          </button>
        </form>

        {/* Demo Fast Login Pills */}
        <div className="pt-4 border-t border-zinc-800/80 space-y-2.5">
          <div className="text-[10px] font-mono text-zinc-500 uppercase tracking-wider text-center">
            Quick Test Accounts
          </div>
          <div className="grid grid-cols-1 gap-2">
            <button
              type="button"
              onClick={() => handleDemoLogin('sarah@acme-corp.com', 'Password123!')}
              className="p-2 rounded-lg bg-zinc-950 hover:bg-zinc-900 border border-zinc-800/80 text-left text-xs transition-colors flex items-center justify-between group"
            >
              <div className="flex items-center space-x-2">
                <Sparkles className="w-3.5 h-3.5 text-emerald-400" />
                <span className="font-mono text-zinc-300 group-hover:text-white">sarah@acme-corp.com</span>
              </div>
              <span className="text-[10px] font-mono text-zinc-500 group-hover:text-zinc-300">Acme Global (Owner) →</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
