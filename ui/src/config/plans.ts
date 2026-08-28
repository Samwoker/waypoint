export interface PlanTier {
  id: 'free' | 'developer' | 'business' | 'enterprise';
  name: string;
  tagline: string;
  priceMonthly: number;
  priceAnnual: number;
  eventLimit: number; // monthly event allowance
  sourceLimit: number;
  destinationLimit: number;
  apiKeyLimit: number;
  logRetentionDays: number;
  rateLimitRps: number;
  features: string[];
  highlight?: boolean;
  badge?: string;
  ctaText: string;
}

export const PLANS: PlanTier[] = [
  {
    id: 'free',
    name: 'Free',
    tagline: 'Ideal for developers prototyping and evaluating RelayCore.',
    priceMonthly: 0,
    priceAnnual: 0,
    eventLimit: 25000,
    sourceLimit: 3,
    destinationLimit: 5,
    apiKeyLimit: 3,
    logRetentionDays: 3,
    rateLimitRps: 50,
    features: [
      '25,000 webhook events / month',
      'Up to 3 Inbound Sources',
      'Up to 5 Target Destinations',
      '3 Scoped API Keys',
      '3-day payload & attempt retention',
      'Exponential backoff retries & jitter',
      'Dead Letter Queue (DLQ) & 1-click replays',
      'HMAC-SHA256, Stripe & GitHub signatures',
      'Community support & public docs',
    ],
    ctaText: 'Start Free',
  },
  {
    id: 'developer',
    name: 'Developer',
    tagline: 'For production microservices and growing SaaS applications.',
    priceMonthly: 29,
    priceAnnual: 24, // $24/mo billed annually
    eventLimit: 250000,
    sourceLimit: 10,
    destinationLimit: 20,
    apiKeyLimit: 10,
    logRetentionDays: 14,
    rateLimitRps: 200,
    features: [
      '250,000 webhook events / month',
      'Up to 10 Inbound Sources',
      'Up to 20 Target Destinations',
      '10 Scoped API Keys',
      '14-day payload & attempt retention',
      'Automated Circuit Breaker protection',
      'JSONPath Payload Transformation engine',
      'Custom retry policies & timeout limits',
      'Standard email support',
    ],
    highlight: true,
    badge: 'MOST POPULAR',
    ctaText: 'Start Building',
  },
  {
    id: 'business',
    name: 'Business',
    tagline: 'For high-throughput mission-critical event infrastructure.',
    priceMonthly: 99,
    priceAnnual: 79, // $79/mo billed annually
    eventLimit: 2500000,
    sourceLimit: 50,
    destinationLimit: 100,
    apiKeyLimit: 50,
    logRetentionDays: 30,
    rateLimitRps: 1000,
    features: [
      '2,500,000 webhook events / month',
      'Up to 50 Inbound Sources',
      'Up to 100 Target Destinations',
      '50 Scoped API Keys',
      '30-day payload & attempt retention',
      'Priority delivery worker queueing',
      'Multi-region routing & advanced filtering',
      'Prometheus telemetry & audit logging',
      'Priority Slack & email support (4h SLA)',
    ],
    ctaText: 'Upgrade to Business',
  },
  {
    id: 'enterprise',
    name: 'Enterprise',
    tagline: 'Custom volume, dedicated clusters, and compliance for enterprises.',
    priceMonthly: 499,
    priceAnnual: 399,
    eventLimit: 50000000,
    sourceLimit: 9999,
    destinationLimit: 9999,
    apiKeyLimit: 9999,
    logRetentionDays: 365,
    rateLimitRps: 10000,
    features: [
      'Custom high-volume event allowances',
      'Unlimited Sources & Destinations',
      'Unlimited API Keys',
      '365-day compliance audit retention',
      'Dedicated isolated worker clusters',
      'Custom SOC2 / HIPAA compliance agreements',
      '99.999% uptime guarantee (SLA)',
      'Dedicated Customer Success Architect',
      '24/7/365 phone & PagerDuty escalation',
    ],
    ctaText: 'Contact Enterprise Sales',
  },
];

export function getPlan(planId: string = 'free'): PlanTier {
  const found = PLANS.find((p) => p.id === planId.toLowerCase());
  return found || PLANS[0];
}

export function formatEventLimit(limit: number): string {
  if (limit >= 1000000) {
    return `${(limit / 1000000).toFixed(limit % 1000000 === 0 ? 0 : 1)}M`;
  }
  if (limit >= 1000) {
    return `${(limit / 1000).toFixed(0)}K`;
  }
  return limit.toLocaleString();
}

export function getUsagePercentage(used: number, limit: number): number {
  if (!limit || limit <= 0) return 0;
  return Math.min(100, Math.round((used / limit) * 100));
}
