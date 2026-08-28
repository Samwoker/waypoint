import React, { useState } from 'react';
import { useLocation, Link } from 'react-router-dom';
import {
  AlertCircle,
  ArrowRight,
  Check,
  CheckCircle2,
  CreditCard,
  Download,
  ExternalLink,
  HelpCircle,
  Receipt,
  ShieldCheck,
  Sparkles,
  Zap,
} from 'lucide-react';
import { useAppSelector } from '../../store/hooks';
import { PLANS, PlanTier, getPlan, formatEventLimit } from '../../config/plans';

export const BillingPage: React.FC = () => {
  const location = useLocation();
  const queryParams = new URLSearchParams(location.search);
  const statusParam = queryParams.get('status');

  const { user } = useAppSelector((state) => state.auth);
  const [currentPlanId, setCurrentPlanId] = useState<'free' | 'developer' | 'business' | 'enterprise'>('free');
  const [isUpgradeModalOpen, setIsUpgradeModalOpen] = useState(false);
  const [selectedTier, setSelectedTier] = useState<PlanTier | null>(null);
  const [isAnnual, setIsAnnual] = useState(true);
  const [isProcessingCheckout, setIsProcessingCheckout] = useState(false);
  const [checkoutSuccess, setCheckoutSuccess] = useState<boolean>(statusParam === 'success');
  const [checkoutFailed, setCheckoutFailed] = useState<boolean>(statusParam === 'failed');

  const activePlan = getPlan(currentPlanId);

  const handleSelectUpgrade = (tier: PlanTier) => {
    setSelectedTier(tier);
    setIsUpgradeModalOpen(true);
  };

  const handleConfirmCheckout = () => {
    if (!selectedTier) return;
    setIsProcessingCheckout(true);

    // Simulate backend checkout session creation / Stripe billing portal redirect
    setTimeout(() => {
      setIsProcessingCheckout(false);
      setCurrentPlanId(selectedTier.id);
      setIsUpgradeModalOpen(false);
      setCheckoutSuccess(true);
      setCheckoutFailed(false);
    }, 1200);
  };

  const sampleInvoices = [
    {
      id: 'inv_88291048',
      date: '2026-08-01',
      plan: 'Free Tier Subscription',
      amount: '$0.00',
      status: 'Paid',
    },
    {
      id: 'inv_77192039',
      date: '2026-07-01',
      plan: 'Free Tier Subscription',
      amount: '$0.00',
      status: 'Paid',
    },
  ];

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-200 font-sans">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-zinc-800 pb-6">
        <div>
          <span className="text-xs font-mono font-semibold uppercase text-emerald-400">
            Subscription & Billing
          </span>
          <h1 className="text-2xl sm:text-3xl font-extrabold text-white tracking-tight mt-1">
            Plans & Invoicing
          </h1>
          <p className="text-xs sm:text-sm text-zinc-400 mt-1">
            Manage your organization subscription tier, payment methods, and invoice history.
          </p>
        </div>

        <button
          type="button"
          onClick={() => handleSelectUpgrade(getPlan('developer'))}
          className="flex items-center space-x-1.5 px-4 py-2 rounded-xl text-xs font-semibold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-md shadow-emerald-500/10 transition-all font-sans"
        >
          <Sparkles className="w-3.5 h-3.5" />
          <span>Change Subscription Plan</span>
        </button>
      </div>

      {/* Success / Failure Banners */}
      {checkoutSuccess && (
        <div className="p-4 rounded-2xl bg-emerald-950/40 border border-emerald-800/80 text-emerald-200 flex items-center justify-between text-xs animate-in fade-in">
          <div className="flex items-center space-x-3">
            <CheckCircle2 className="w-5 h-5 text-emerald-400 shrink-0" />
            <div>
              <strong className="block font-bold">Subscription Successfully Updated!</strong>
              <span>Your account is now active on the {activePlan.name} plan. All increased limits take effect immediately.</span>
            </div>
          </div>
          <button
            type="button"
            onClick={() => setCheckoutSuccess(false)}
            className="px-3 py-1 bg-emerald-500/20 hover:bg-emerald-500/30 rounded-lg text-emerald-300 font-mono text-[11px]"
          >
            Dismiss
          </button>
        </div>
      )}

      {checkoutFailed && (
        <div className="p-4 rounded-2xl bg-rose-950/40 border border-rose-800/80 text-rose-200 flex items-center justify-between text-xs animate-in fade-in">
          <div className="flex items-center space-x-3">
            <AlertCircle className="w-5 h-5 text-rose-400 shrink-0" />
            <div>
              <strong className="block font-bold">Payment Could Not Be Completed</strong>
              <span>Your subscription remains unchanged. Please try again or use another payment method.</span>
            </div>
          </div>
          <button
            type="button"
            onClick={() => setCheckoutFailed(false)}
            className="px-3 py-1 bg-rose-500/20 hover:bg-rose-500/30 rounded-lg text-rose-300 font-mono text-[11px]"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Current Subscription Card */}
      <div className="p-6 rounded-3xl bg-[#0e0e11] border border-zinc-800 space-y-6 shadow-sm">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-zinc-800/80 pb-6">
          <div className="flex items-start space-x-4">
            <div className="w-12 h-12 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center font-bold">
              <Zap className="w-6 h-6" />
            </div>
            <div className="space-y-1">
              <div className="flex items-center space-x-2">
                <h2 className="text-lg font-bold text-white">{activePlan.name} Plan</h2>
                <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                  ACTIVE
                </span>
              </div>
              <p className="text-xs text-zinc-400 max-w-md">{activePlan.tagline}</p>
            </div>
          </div>

          <div className="text-right sm:text-right">
            <div className="flex items-baseline justify-start sm:justify-end space-x-1">
              <span className="text-3xl font-extrabold text-white">
                ${activePlan.priceMonthly}
              </span>
              <span className="text-xs text-zinc-500">/month</span>
            </div>
            <span className="text-[11px] font-mono text-zinc-500 block mt-0.5">
              Renews automatically on Oct 1, 2026
            </span>
          </div>
        </div>

        {/* Quota Highlights */}
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 text-xs">
          <div className="p-3.5 rounded-xl bg-zinc-950 border border-zinc-800/80 space-y-1">
            <span className="text-[10px] font-mono uppercase text-zinc-500 block">Monthly Events</span>
            <strong className="text-white text-sm font-mono">{formatEventLimit(activePlan.eventLimit)}</strong>
          </div>
          <div className="p-3.5 rounded-xl bg-zinc-950 border border-zinc-800/80 space-y-1">
            <span className="text-[10px] font-mono uppercase text-zinc-500 block">Destinations</span>
            <strong className="text-white text-sm font-mono">{activePlan.destinationLimit} Endpoints</strong>
          </div>
          <div className="p-3.5 rounded-xl bg-zinc-950 border border-zinc-800/80 space-y-1">
            <span className="text-[10px] font-mono uppercase text-zinc-500 block">Sources</span>
            <strong className="text-white text-sm font-mono">{activePlan.sourceLimit} Inbound</strong>
          </div>
          <div className="p-3.5 rounded-xl bg-zinc-950 border border-zinc-800/80 space-y-1">
            <span className="text-[10px] font-mono uppercase text-zinc-500 block">Payload Retention</span>
            <strong className="text-white text-sm font-mono">{activePlan.logRetentionDays} Days</strong>
          </div>
        </div>
      </div>

      {/* Available Plans Selection Grid */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-bold text-white">Available Upgrade Plans</h3>
          <Link to="/pricing" className="text-xs text-emerald-400 hover:underline flex items-center space-x-1">
            <span>View public pricing & features</span>
            <ExternalLink className="w-3 h-3" />
          </Link>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {PLANS.filter((p) => p.id !== 'enterprise').map((plan) => {
            const isCurrent = plan.id === currentPlanId;

            return (
              <div
                key={plan.id}
                className={`p-6 rounded-3xl bg-[#0e0e11] border transition-all flex flex-col justify-between space-y-6 ${
                  isCurrent
                    ? 'border-emerald-500/80 shadow-md'
                    : 'border-zinc-800/80 hover:border-zinc-700'
                }`}
              >
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <h4 className="text-base font-bold text-white">{plan.name}</h4>
                    {isCurrent && (
                      <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                        CURRENT PLAN
                      </span>
                    )}
                  </div>

                  <div className="flex items-baseline space-x-1">
                    <span className="text-3xl font-extrabold text-white">${plan.priceMonthly}</span>
                    <span className="text-xs text-zinc-500">/month</span>
                  </div>

                  <p className="text-xs text-zinc-400 leading-relaxed min-h-[32px]">
                    {plan.tagline}
                  </p>

                  <ul className="space-y-2 pt-2 border-t border-zinc-800/80 text-xs text-zinc-300">
                    {plan.features.slice(0, 4).map((f, idx) => (
                      <li key={idx} className="flex items-start space-x-2">
                        <Check className="w-3.5 h-3.5 text-emerald-400 shrink-0 mt-0.5" />
                        <span className="leading-tight">{f}</span>
                      </li>
                    ))}
                  </ul>
                </div>

                <button
                  type="button"
                  disabled={isCurrent}
                  onClick={() => handleSelectUpgrade(plan)}
                  className={`w-full py-2.5 px-4 rounded-xl text-xs font-semibold transition-all ${
                    isCurrent
                      ? 'bg-zinc-800 text-zinc-500 cursor-not-allowed'
                      : 'bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-bold shadow-md shadow-emerald-500/10'
                  }`}
                >
                  {isCurrent ? 'Current Plan' : `Upgrade to ${plan.name}`}
                </button>
              </div>
            );
          })}
        </div>
      </div>

      {/* Invoices History Table */}
      <div className="p-6 rounded-3xl bg-[#0e0e11] border border-zinc-800 space-y-4 shadow-sm">
        <div className="flex items-center space-x-2">
          <Receipt className="w-4 h-4 text-emerald-400" />
          <h3 className="text-sm font-bold text-white">Billing & Invoice History</h3>
        </div>

        <div className="overflow-x-auto rounded-2xl border border-zinc-800/80">
          <table className="w-full text-left text-xs">
            <thead>
              <tr className="border-b border-zinc-800 bg-zinc-950/80 font-mono text-[10px] uppercase text-zinc-500">
                <th className="py-3 px-4 font-semibold">Invoice Date</th>
                <th className="py-3 px-4 font-semibold">Invoice ID</th>
                <th className="py-3 px-4 font-semibold">Description</th>
                <th className="py-3 px-4 font-semibold">Amount</th>
                <th className="py-3 px-4 font-semibold">Status</th>
                <th className="py-3 px-4 font-semibold text-right">Receipt</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/50 font-mono text-zinc-300">
              {sampleInvoices.map((inv) => (
                <tr key={inv.id} className="hover:bg-zinc-900/40">
                  <td className="py-3 px-4 text-zinc-400">{inv.date}</td>
                  <td className="py-3 px-4 font-bold text-white">{inv.id}</td>
                  <td className="py-3 px-4">{inv.plan}</td>
                  <td className="py-3 px-4 font-bold text-emerald-400">{inv.amount}</td>
                  <td className="py-3 px-4">
                    <span className="px-2 py-0.5 rounded-full text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                      {inv.status}
                    </span>
                  </td>
                  <td className="py-3 px-4 text-right">
                    <button
                      type="button"
                      onClick={() => alert(`Downloading invoice ${inv.id}`)}
                      className="text-zinc-400 hover:text-white p-1 hover:bg-zinc-800 rounded transition-colors"
                      title="Download PDF"
                    >
                      <Download className="w-3.5 h-3.5" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Upgrade Checkout Modal */}
      {isUpgradeModalOpen && selectedTier && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in">
          <div className="w-full max-w-lg bg-[#0e0e11] border border-zinc-800 rounded-3xl p-6 sm:p-8 space-y-6 shadow-2xl animate-in zoom-in-95">
            <div className="space-y-2 text-center">
              <div className="w-10 h-10 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center mx-auto">
                <Sparkles className="w-5 h-5" />
              </div>
              <h3 className="text-xl font-extrabold text-white tracking-tight">
                Upgrade to {selectedTier.name} Plan
              </h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Unlock {formatEventLimit(selectedTier.eventLimit)} monthly events, automated circuit breakers, and extended retention.
              </p>
            </div>

            {/* Price Summary Card */}
            <div className="p-4 rounded-2xl bg-zinc-950 border border-zinc-800 flex items-center justify-between">
              <div>
                <span className="text-xs font-bold text-white block">{selectedTier.name} Monthly Subscription</span>
                <span className="text-[11px] font-mono text-zinc-500">Prorated and active immediately</span>
              </div>
              <div className="text-right font-mono">
                <span className="text-xl font-extrabold text-emerald-400">${selectedTier.priceMonthly}</span>
                <span className="text-xs text-zinc-500">/mo</span>
              </div>
            </div>

            <div className="space-y-2 text-xs text-zinc-300">
              <span className="font-semibold block font-mono uppercase text-[10px] text-zinc-500">Included Features:</span>
              <ul className="space-y-1.5 pl-2">
                {selectedTier.features.slice(0, 4).map((f, i) => (
                  <li key={i} className="flex items-center space-x-2">
                    <Check className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                    <span>{f}</span>
                  </li>
                ))}
              </ul>
            </div>

            {/* Actions */}
            <div className="flex items-center space-x-3 pt-2">
              <button
                type="button"
                onClick={() => setIsUpgradeModalOpen(false)}
                className="flex-1 py-2.5 px-4 rounded-xl text-xs font-semibold bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-300 transition-colors"
              >
                Cancel
              </button>

              <button
                type="button"
                disabled={isProcessingCheckout}
                onClick={handleConfirmCheckout}
                className="flex-1 py-2.5 px-4 rounded-xl text-xs font-bold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-lg shadow-emerald-500/10 transition-all flex items-center justify-center space-x-2 disabled:opacity-50"
              >
                {isProcessingCheckout ? (
                  <span>Processing...</span>
                ) : (
                  <>
                    <span>Confirm & Activate</span>
                    <ArrowRight className="w-4 h-4" />
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
