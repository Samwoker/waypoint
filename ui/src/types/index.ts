export interface Tenant {
  id: string;
  name: string;
  slug: string;
  created_at: string;
  updated_at: string;
}

export interface TenantUsage {
  total_events: number;
  total_delivery_attempts: number;
  daily_events: [string, number][];
}

export interface Source {
  id: string;
  tenant_id: string;
  name: string;
  slug: string;
  description?: string;
  provider: string; // 'stripe' | 'github' | 'shopify' | 'generic'
  verification_type: string;
  is_active: boolean;
  has_secret: boolean;
  timestamp_tolerance_secs?: number;
  created_at: string;
  updated_at: string;
}

export interface VerificationLog {
  received_at: string;
  signature_valid: boolean;
  external_event_id?: string;
}

export interface Destination {
  id: string;
  tenant_id: string;
  name: string;
  url: string;
  description?: string;
  rate_limit?: number;
  timeout_ms: number;
  max_retry_count: number;
  initial_backoff_sec: number;
  is_active: boolean;
  has_secret: boolean;
  consecutive_failures: number;
  circuit_status: 'closed' | 'open' | 'half_open';
  circuit_opened_at?: string;
  created_at: string;
  updated_at: string;
}

export interface Subscription {
  id: string;
  tenant_id: string;
  source_id: string;
  destination_id: string;
  source_name?: string;
  destination_name?: string;
  event_types: string[];
  filter_expression?: string;
  retry_policy?: Record<string, any>;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface EventItem {
  id: string;
  tenant_id: string;
  source_id?: string;
  event_type: string;
  payload: any;
  headers?: Record<string, any>;
  idempotency_key?: string;
  created_at: string;
}

export interface Delivery {
  id: string;
  tenant_id: string;
  event_id: string;
  subscription_id: string;
  destination_id: string;
  destination_name?: string;
  destination_url?: string;
  event_type?: string;
  status: 'pending' | 'delivered' | 'failed' | 'dead_letter' | 'discarded';
  attempt_count: number;
  max_attempts: number;
  next_retry_at?: string;
  last_error?: string;
  created_at: string;
  updated_at: string;
}

export interface DeliveryAttempt {
  id: string;
  delivery_id: string;
  attempt_number: number;
  status: 'success' | 'failed';
  response_status?: number;
  request_headers?: Record<string, any>;
  request_body?: string;
  response_headers?: Record<string, any>;
  response_body?: string;
  error_message?: string;
  duration_ms?: number;
  created_at: string;
}

export interface DlqRecord {
  delivery_id: string;
  tenant_id: string;
  event_id: string;
  event_type: string;
  destination_id: string;
  destination_name: string;
  destination_url: string;
  status: string;
  attempt_count: number;
  max_attempts: number;
  last_error?: string;
  created_at: string;
  updated_at: string;
}

export interface TransformationRule {
  source_path: string;
  dest_path: string;
}

export interface Transformation {
  id: string;
  tenant_id: string;
  subscription_id: string;
  rules: TransformationRule[];
  created_at: string;
  updated_at: string;
}

export interface SystemStats {
  total_tenants: number;
  active_tenants: number;
  total_sources: number;
  total_destinations: number;
  total_subscriptions: number;
  total_events: number;
  total_deliveries: number;
  successful_deliveries: number;
  failed_deliveries: number;
  dead_letter_deliveries: number;
}

export interface OverviewStats {
  total_events: number;
  total_deliveries: number;
  delivered_count: number;
  success_rate: number;
  p50_latency_ms?: number;
  p95_latency_ms?: number;
}

export interface TimeseriesPoint {
  bucket: string;
  value: number;
}

export interface ApiKey {
  id: string;
  name: string;
  key_prefix: string;
  is_active: boolean;
  expires_at?: string;
  last_used_at?: string;
  created_at: string;
}

export interface ApiKeyCreated {
  id: string;
  name: string;
  key: string;
  key_prefix: string;
  expires_at?: string;
  created_at: string;
}

export interface User {
  id: string;
  tenant_id: string;
  email: string;
  role: string;
  is_admin: boolean;
  status: string;
  created_at: string;
}
