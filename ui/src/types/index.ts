export interface Tenant {
  id: string;
  name: string;
  slug: string;
  created_at: string;
  updated_at: string;
}

export interface DailyEventCount {
  date: string;
  count: number;
}

export interface TenantUsage {
  tenant_id: string;
  period: string;
  total_events: number;
  total_delivery_attempts: number;
  daily_events: DailyEventCount[];
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
  secret?: string;
  created_at: string;
  updated_at: string;
}

export interface RotateSecretResponse {
  source_id: string;
  secret: string;
  warning: string;
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
  rate_limit_rps?: number;
  timeout_ms: number;
  max_retries: number;
  retry_backoff_strategy?: string;
  is_active: boolean;
  status: string; // 'active' | 'paused' | 'circuit_open' | 'deleted'
  consecutive_failures: number;
  circuit_opened_at?: string;
  secret?: string;
  created_at: string;
  updated_at: string;
}

export interface DestinationHealth {
  status: string;
  consecutive_failures: number;
  circuit_opened_at?: string;
  success_rate: number;
  total_attempts: number;
  successful_attempts: number;
}

export interface TestDestinationResponse {
  success: boolean;
  http_status?: number;
  latency_ms: number;
  error?: string;
}

export interface Subscription {
  id: string;
  tenant_id: string;
  source_id: string;
  destination_id: string;
  source_name?: string;
  destination_name?: string;
  event_types: string[];
  filter_rules?: any;
  transformation_template?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface EventItem {
  id: string;
  tenant_id: string;
  source_id?: string;
  event_type: string;
  idempotency_key?: string;
  status: string;
  received_at: string;
  created_at: string;
}

export interface DeliverySummary {
  total: number;
  delivered: number;
  failed: number;
  pending: number;
}

export interface EventDetail {
  id: string;
  tenant_id: string;
  source_id: string;
  event_type: string;
  idempotency_key?: string;
  status: string;
  delivery_summary: DeliverySummary;
  received_at: string;
  created_at: string;
}

export interface EventDeliveryItem {
  id: string;
  destination_id: string;
  destination_name: string;
  status: string;
  attempt_count: number;
  next_attempt_at?: string;
  delivered_at?: string;
  created_at: string;
}

export interface RawEventPayload {
  event_id: string;
  headers: Record<string, any>;
  payload: string;
  is_binary: boolean;
}

export interface PaginatedEvents {
  events: EventItem[];
  next_cursor?: string;
  has_more: boolean;
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
  status: 'pending' | 'delivered' | 'failed' | 'dead_letter' | 'discarded' | string;
  attempt_count: number;
  max_attempts: number;
  next_retry_at?: string;
  created_at: string;
  updated_at: string;
}

export interface DeliveryAttempt {
  id: string;
  attempt_number: number;
  http_status?: number;
  response_body_snippet?: string;
  latency_ms?: number;
  error_message?: string;
  created_at: string;
}

export interface DeliveryDetail {
  id: string;
  tenant_id: string;
  event_id: string;
  subscription_id: string;
  destination_id: string;
  status: string;
  attempt_count: number;
  max_attempts: number;
  next_retry_at?: string;
  attempts: DeliveryAttempt[];
  created_at: string;
  updated_at: string;
}

export interface PaginatedDeliveries {
  deliveries: Delivery[];
  next_cursor?: string;
  has_more: boolean;
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

export interface PaginatedDlq {
  items: DlqRecord[];
  next_cursor?: string;
  has_more: boolean;
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
  tenant_id: string;
  name: string;
  key_prefix: string;
  expires_at?: string;
  last_used_at?: string;
  created_at: string;
}

export interface ApiKeyCreated {
  id: string;
  name: string;
  raw_key: string;
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
  status?: string;
  created_at: string;
}
