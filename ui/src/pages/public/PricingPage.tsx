import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import {
  ArrowRight,
  Check,
  HelpCircle,
  ShieldCheck,
  Sparkles,
  X,
  Zap,
} from 'lucide-react';
import { PublicLayout } from '../../components/public/PublicLayout';
import { PLANS, formatEventLimit } from '../../config/plans';
import { useAppSelector } from '../../store/hooks';

export const PricingPage: React.FC = () => {
  const { user, token } = useAppSelector((state) => state.auth);
  const isAuthenticated = !!(user && token);
  const [isAnnual, setIsAnnual] = useState(true);

  const faqs = [
    {
      q: 'What happens when I reach my monthly event limit on the Free tier?',
      a: 'We will notify you by email and in your dashboard when you reach 80% and 100% of your event allowance. You can upgrade your plan at any time to immediately continue ingesting events with zero dropped payloads.',
    },
    {
      q: 'Do I need a credit card to sign up for the Free plan?',
      a: 'No. The Free plan requires only your email address and organization name. You can use all core platform features (HMAC verification, DLQ, retries) for free.',
    },
    {
      q: 'Can I upgrade, downgrade, or cancel at any time?',
      a: 'Yes. Upgrades take effect immediately and are prorated. Downgrades or cancellations take effect at the end of your current billing cycle.',
    },
    {
      q: 'How does RelayCore calculate event volume?',
      a: 'An event is counted when an incoming webhook is accepted at `/hooks/:slug`. Outbound delivery attempts and retries do not count against your inbound event quota.',
    },
    {
      q: 'Does RelayCore offer custom enterprise SLAs?',
      a: 'Yes. Our Enterprise plan includes custom 99.999% uptime SLAs, dedicated worker infrastructure, SOC2/HIPAA compliance agreements, and dedicated solution architects.',
    },
  ];

  return (
    <PublicLayout>
      <div className="space-y-20 py-12 sm:py-16 max-w-7xl mx-auto px-4 sm:px-8 animate-in fade-in duration-200">
        {/* Header */}
        <div className="text-center space-y-4 max-w-3xl mx-auto">
          <div className="inline-flex items-center space-x-2 px-3 py-1 rounded-full text-xs font-mono font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            <Zap className="w-3.5 h-3.5" />
            <span>Transparent Infrastructure Pricing</span>
          </div>

          <h1 className="text-3xl sm:text-5xl font-extrabold text-white tracking-tight">
            Simple, Predictable Pricing for Developers
          </h1>

          <p className="text-sm sm:text-base text-zinc-400 leading-relaxed">
            Start with our generous Free Tier. Scale seamlessly to millions of events as your production application expands.
          </p>

          {/* Monthly / Annual Toggle */}
          <div className="flex items-center justify-center space-x-3 pt-4">
            <span className={`text-xs font-medium ${!isAnnual ? 'text-white' : 'text-zinc-500'}`}>
              Monthly Billing
            </span>
            <button
              type="button"
              onClick={() => setIsAnnual(!isAnnual)}
              className="w-12 h-6 rounded-full bg-zinc-800 p-1 transition-colors relative border border-zinc-700 focus:outline-none"
              aria-label="Toggle annual billing"
            >
              <div
                className={`w-4 h-4 rounded-full bg-emerald-400 transition-transform ${
                  isAnnual ? 'translate-x-6' : 'translate-x-0'
                }`}
              />
            </button>
            <span className={`text-xs font-medium flex items-center space-x-1.5 ${isAnnual ? 'text-white' : 'text-zinc-500'}`}>
              <span>Annual Billing</span>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                SAVE 20%
              </span>
            </span>
          </div>
        </div>

        {/* Pricing Cards Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {PLANS.map((plan) => {
            const price = isAnnual ? plan.priceAnnual : plan.priceMonthly;

            return (
              <div
                key={plan.id}
                className={`p-6 rounded-3xl bg-[#0e0e11] border transition-all flex flex-col justify-between space-y-6 ${
                  plan.highlight
                    ? 'border-emerald-500 shadow-2xl shadow-emerald-500/10 relative'
                    : 'border-zinc-800/80 hover:border-zinc-700'
                }`}
              >
                {plan.badge && (
                  <span className="absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-0.5 rounded-full text-[10px] font-mono font-bold bg-emerald-500 text-zinc-950 uppercase tracking-wider shadow-sm">
                    {plan.badge}
                  </span>
                )}

                <div className="space-y-4">
                  <div className="space-y-1">
                    <h3 className="text-base font-bold text-white">{plan.name}</h3>
                    <p className="text-xs text-zinc-400 leading-relaxed min-h-[32px]">
                      {plan.tagline}
                    </p>
                  </div>

                  <div className="flex items-baseline space-x-1">
                    <span className="text-3xl font-extrabold text-white">
                      ${price}
                    </span>
                    <span className="text-xs text-zinc-500">
                      {plan.id === 'enterprise' ? '/custom' : '/month'}
                    </span>
                  </div>

                  <div className="p-3 rounded-xl bg-zinc-950 border border-zinc-800/80 space-y-1 text-[11px] font-mono text-zinc-400">
                    <div className="flex justify-between">
                      <span>Event Allowance:</span>
                      <strong className="text-white">{formatEventLimit(plan.eventLimit)}</strong>
                    </div>
                    <div className="flex justify-between">
                      <span>Log Retention:</span>
                      <strong className="text-white">{plan.logRetentionDays} days</strong>
                    </div>
                  </div>

                  <ul className="space-y-2 pt-2 text-xs text-zinc-300">
                    {plan.features.map((f, idx) => (
                      <li key={idx} className="flex items-start space-x-2">
                        <Check className="w-3.5 h-3.5 text-emerald-400 shrink-0 mt-0.5" />
                        <span className="leading-tight">{f}</span>
                      </li>
                    ))}
                  </ul>
                </div>

                <Link
                  to={
                    isAuthenticated
                      ? '/dashboard/billing'
                      : plan.id === 'free'
                      ? '/signup'
                      : `/signup?plan=${plan.id}`
                  }
                  className={`w-full py-2.5 px-4 rounded-xl text-xs font-semibold text-center transition-all ${
                    plan.highlight
                      ? 'bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-bold shadow-md'
                      : 'bg-zinc-800 hover:bg-zinc-700 text-white'
                  }`}
                >
                  {plan.ctaText}
                </Link>
              </div>
            );
          })}
        </div>

        {/* FEATURE COMPARISON TABLE */}
        <div className="space-y-6 pt-12">
          <div className="text-center space-y-2">
            <h2 className="text-2xl font-bold text-white tracking-tight">
              Detailed Feature Comparison
            </h2>
            <p className="text-xs text-zinc-400">
              Compare platform quotas, security controls, and infrastructure guarantees.
            </p>
          </div>

          <div className="overflow-x-auto rounded-2xl border border-zinc-800 bg-[#0e0e11] shadow-lg">
            <table className="w-full text-left text-xs border-collapse font-sans">
              <thead>
                <tr className="border-b border-zinc-800 bg-zinc-950 font-mono text-[11px] uppercase text-zinc-400">
                  <th className="py-4 px-5 font-semibold">Capability</th>
                  <th className="py-4 px-5 font-semibold">Free</th>
                  <th className="py-4 px-5 font-semibold text-emerald-400">Developer</th>
                  <th className="py-4 px-5 font-semibold">Business</th>
                  <th className="py-4 px-5 font-semibold">Enterprise</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/60 text-zinc-300">
                <tr>
                  <td className="py-3.5 px-5 font-medium text-white">Monthly Events</td>
                  <td className="py-3.5 px-5 font-mono">25,000</td>
                  <td className="py-3.5 px-5 font-mono text-emerald-400 font-semibold">250,000</td>
                  <td className="py-3.5 px-5 font-mono">2,500,000</td>
                  <td className="py-3.5 px-5 font-mono">Unlimited</td>
                </tr>
                <tr>
                  <td className="py-3.5 px-5 font-medium text-white">Inbound Sources</td>
                  <td className="py-3.5 px-5">3</td>
                  <td className="py-3.5 px-5 text-emerald-400 font-semibold">10</td>
                  <td className="py-3.5 px-5">50</td>
                  <td className="py-3.5 px-5">Unlimited</td>
                </tr>
                <tr>
                  <td className="py-3.5 px-5 font-medium text-white">Target Destinations</td>
                  <td className="py-3.5 px-5">5</td>
                  <td className="py-3.5 px-5 text-emerald-400 font-semibold">20</td>
                  <td className="py-3.5 px-5">100</td>
                  <td className="py-3.5 px-5">Unlimited</td>
                </tr>
                <tr>
                  <td className="py-3.5 px-5 font-medium text-white">HMAC Signatures (Stripe/GitHub)</td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                </tr>
                <tr>
                  <td className="py-3.5 px-5 font-medium text-white">Automated Circuit Breakers</td>
                  <td className="py-3.5 px-5"><X className="w-4 h-4 text-zinc-600" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                </tr>
                <tr>
                  <td className="py-3.5 px-5 font-medium text-white">JSONPath Transformations</td>
                  <td className="py-3.5 px-5"><X className="w-4 h-4 text-zinc-600" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                </tr>
                <tr>
                  <td className="py-3.5 px-5 font-medium text-white">Dead Letter Queue & Replay</td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                  <td className="py-3.5 px-5"><Check className="w-4 h-4 text-emerald-400" /></td>
                </tr>
                <tr>
                  <td className="py-3.5 px-5 font-medium text-white">Log Retention</td>
                  <td className="py-3.5 px-5">3 days</td>
                  <td className="py-3.5 px-5 text-emerald-400 font-semibold">14 days</td>
                  <td className="py-3.5 px-5">30 days</td>
                  <td className="py-3.5 px-5">365 days</td>
                </tr>
                <tr>
                  <td className="py-3.5 px-5 font-medium text-white">SLA Guarantee</td>
                  <td className="py-3.5 px-5 text-zinc-500">Best effort</td>
                  <td className="py-3.5 px-5">99.9%</td>
                  <td className="py-3.5 px-5">99.95%</td>
                  <td className="py-3.5 px-5 text-emerald-400 font-semibold">99.999%</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        {/* FREQUENTLY ASKED QUESTIONS */}
        <div className="space-y-8 pt-12 max-w-4xl mx-auto">
          <div className="text-center space-y-2">
            <h2 className="text-2xl font-bold text-white tracking-tight">
              Pricing Frequently Asked Questions
            </h2>
            <p className="text-xs text-zinc-400">
              Clear answers regarding billing cycles, limits, and upgrades.
            </p>
          </div>

          <div className="space-y-4">
            {faqs.map((faq, idx) => (
              <div key={idx} className="p-5 rounded-2xl bg-[#0e0e11] border border-zinc-800/80 space-y-2">
                <h4 className="text-sm font-bold text-white flex items-center space-x-2">
                  <HelpCircle className="w-4 h-4 text-emerald-400 shrink-0" />
                  <span>{faq.q}</span>
                </h4>
                <p className="text-xs text-zinc-400 leading-relaxed pl-6">
                  {faq.a}
                </p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </PublicLayout>
  );
};
