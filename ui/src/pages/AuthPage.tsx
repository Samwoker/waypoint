import React, { useEffect, useState } from 'react';
import { useLocation, useNavigate, Link } from 'react-router-dom';
import {
  AlertCircle,
  ArrowRight,
  Building,
  CheckCircle2,
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
  const location = useLocation();
  const dispatch = useAppDispatch();
  const { user, token, isLoading, error } = useAppSelector((state) => state.auth);

  // Extract redirect query parameter: e.g. /login?redirect=/dashboard/api-keys
  const queryParams = new URLSearchParams(location.search);
  const redirectUrl = queryParams.get('redirect') || '/dashboard';
  const isSignupPath = location.pathname === '/signup';

  const [mode, setMode] = useState<'signin' | 'register'>(isSignupPath ? 'register' : 'signin');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [tenantName, setTenantName] = useState('');

  // Update mode when path changes
  useEffect(() => {
    setMode(location.pathname === '/signup' ? 'register' : 'signin');
    dispatch(clearAuthError());
  }, [location.pathname, dispatch]);

  // If already logged in, redirect to destination
  useEffect(() => {
    if (user && token) {
      navigate(redirectUrl, { replace: true });
    }
  }, [user, token, navigate, redirectUrl]);

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

  const handleGoogleSignIn = () => {
    // Initiate Google OAuth flow: in production, redirect to backend OAuth endpoint or Google OAuth URL
    // Simulate real Google login callback or redirect
    const demoGoogleEmail = 'google.developer@example.com';
    const demoGooglePass = 'GoogleSecurePass123!';
    dispatch(loginRequest({ email: demoGoogleEmail, pass: demoGooglePass }));
  };

  return (
    <div className="min-h-screen bg-[#09090b] flex flex-col justify-center items-center p-6 bg-grid-pattern relative overflow-hidden font-sans">
      {/* Background ambient lighting */}
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-96 h-96 bg-emerald-500/10 rounded-full blur-3xl pointer-events-none" />
      <div className="absolute bottom-1/4 right-1/4 w-80 h-80 bg-blue-500/10 rounded-full blur-3xl pointer-events-none" />

      {/* Brand Navigation Link */}
      <Link to="/" className="flex items-center space-x-2 mb-8 group z-10">
        <div className="w-8 h-8 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center font-bold font-mono group-hover:scale-105 transition-transform">
          <Zap className="w-4 h-4" />
        </div>
        <span className="font-bold text-white text-lg tracking-tight">RelayCore</span>
      </Link>

      {/* Main Authentication Card */}
      <div className="w-full max-w-md bg-[#121215]/95 border border-zinc-800 backdrop-blur-xl rounded-3xl p-8 shadow-2xl space-y-6 relative z-10 animate-in fade-in zoom-in-95 duration-200">
        {/* Header */}
        <div className="text-center space-y-1.5">
          <h1 className="text-2xl font-extrabold text-white tracking-tight">
            {mode === 'signin' ? 'Welcome Back' : 'Create Your Account'}
          </h1>
          <p className="text-xs text-zinc-400">
            {mode === 'signin'
              ? 'Sign in to manage your webhook pipelines and deliveries.'
              : 'Start building with 25,000 free events on the Free Tier.'}
          </p>
        </div>

        {/* Tab Switcher */}
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
            Sign Up Free
          </button>
        </div>

        {/* Google Sign In Button */}
        <button
          type="button"
          onClick={handleGoogleSignIn}
          className="w-full py-2.5 px-4 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-700 text-zinc-200 hover:text-white font-medium text-xs transition-all flex items-center justify-center space-x-2.5 shadow-sm active:scale-98"
        >
          <svg className="w-4 h-4" viewBox="0 0 24 24">
            <path
              fill="#EA4335"
              d="M12 5c1.5 0 2.8.5 3.9 1.5l2.9-2.9C17 2 14.6 1 12 1 7.5 1 3.7 3.6 1.9 7.3l3.6 2.8C6.4 7.2 8.9 5 12 5z"
            />
            <path
              fill="#4285F4"
              d="M23.5 12.3c0-.8-.1-1.6-.2-2.3H12v4.6h6.5c-.3 1.5-1.1 2.8-2.4 3.7l3.7 2.9c2.2-2 3.7-5 3.7-8.9z"
            />
            <path
              fill="#FBBC05"
              d="M5.5 14.9c-.3-.8-.4-1.8-.4-2.9s.2-2 .4-2.9L1.9 6.3C.7 8.7 0 10.3 0 12s.7 3.3 1.9 5.7l3.6-2.8z"
            />
            <path
              fill="#34A853"
              d="M12 23c3.2 0 6-1.1 8-3l-3.7-2.9c-1.1.7-2.5 1.2-4.3 1.2-3.1 0-5.6-2.2-6.5-5.1L1.9 16C3.7 19.7 7.5 23 12 23z"
            />
          </svg>
          <span>Continue with Google</span>
        </button>

        {/* Divider */}
        <div className="relative flex items-center justify-center">
          <div className="border-t border-zinc-800 w-full" />
          <span className="bg-[#121215] px-3 text-[10px] font-mono uppercase text-zinc-500 relative">
            Or with email
          </span>
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
              <label className="block text-xs font-mono text-zinc-400">
                Organization / Workspace Name
              </label>
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
            className="w-full py-2.5 px-4 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-bold text-xs transition-all shadow-md shadow-emerald-500/10 active:scale-98 disabled:opacity-50 flex items-center justify-center space-x-2"
          >
            {isLoading ? (
              <Loader2 className="w-4 h-4 animate-spin text-zinc-950" />
            ) : (
              <>
                <span>
                  {mode === 'signin' ? 'Sign In to Dashboard' : 'Create Free Organization & Enter'}
                </span>
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
                <span className="font-mono text-zinc-300 group-hover:text-white">
                  sarah@acme-corp.com
                </span>
              </div>
              <span className="text-[10px] font-mono text-zinc-500 group-hover:text-zinc-300">
                Acme Global (Free Plan) →
              </span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
