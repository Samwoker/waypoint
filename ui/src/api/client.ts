import {
  ApiKey,
  ApiKeyCreated,
  Delivery,
  DeliveryAttempt,
  Destination,
  DlqRecord,
  EventItem,
  OverviewStats,
  Source,
  Subscription,
  SystemStats,
  Tenant,
  TenantUsage,
  TimeseriesPoint,
  Transformation,
  User,
  VerificationLog,
} from '../types';

const API_BASE = ''; // Proxied through Vite to http://localhost:3001

class ApiService {
  private token: string | null = null;
  private apiKey: string | null = null;

  constructor() {
    this.token = localStorage.getItem('waypoint_token');
    this.apiKey = localStorage.getItem('waypoint_api_key');
  }

  setToken(token: string | null) {
    this.token = token;
    if (token) {
      localStorage.setItem('waypoint_token', token);
    } else {
      localStorage.removeItem('waypoint_token');
    }
  }

  setApiKey(key: string | null) {
    this.apiKey = key;
    if (key) {
      localStorage.setItem('waypoint_api_key', key);
    } else {
      localStorage.removeItem('waypoint_api_key');
    }
  }

  getToken() {
    return this.token;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string>),
    };

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    } else if (this.apiKey) {
      headers['X-Api-Key'] = this.apiKey;
    }

    const response = await fetch(`${API_BASE}${endpoint}`, {
      ...options,
      headers,
    });

    if (response.status === 204) {
      return {} as T;
    }

    const data = await response.json();
    if (!response.ok) {
      throw new Error(data.error?.message || data.message || `Request failed with status ${response.status}`);
    }

    return data as T;
  }

  // --- Auth & User ---
  async register(email: string, password: string, tenantName: string): Promise<{ access_token: string; refresh_token: string }> {
    const res = await this.request<{ access_token: string; refresh_token: string }>('/api/v1/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password, tenant_name: tenantName }),
    });
    this.setToken(res.access_token);
    return res;
  }

  async login(email: string, password: string): Promise<{ access_token: string; refresh_token: string }> {
    const res = await this.request<{ access_token: string; refresh_token: string }>('/api/v1/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });
    this.setToken(res.access_token);
    return res;
  }

  async getMe(): Promise<User> {
    return this.request<User>('/api/v1/auth/me');
  }

  // --- Tenants ---
  async listTenants(): Promise<Tenant[]> {
    return this.request<Tenant[]>('/api/v1/tenants');
  }

  async createTenant(name: string, slug: string): Promise<Tenant> {
    return this.request<Tenant>('/api/v1/tenants', {
      method: 'POST',
      body: JSON.stringify({ name, slug }),
    });
  }

  async getTenantUsage(tenantId: string, period?: string): Promise<TenantUsage> {
    const query = period ? `?period=${period}` : '';
    return this.request<TenantUsage>(`/api/v1/tenants/${tenantId}/usage${query}`);
  }

  // --- Sources ---
  async listSources(): Promise<Source[]> {
    return this.request<Source[]>('/api/v1/sources');
  }

  async createSource(input: {
    name: string;
    slug: string;
    description?: string;
    provider: string;
    verification_type: string;
    secret?: string;
  }): Promise<Source> {
    return this.request<Source>('/api/v1/sources', {
      method: 'POST',
      body: JSON.stringify(input),
    });
  }

  async updateSource(id: string, input: Partial<Source>): Promise<Source> {
    return this.request<Source>(`/api/v1/sources/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(input),
    });
  }

  async deleteSource(id: string): Promise<void> {
    return this.request<void>(`/api/v1/sources/${id}`, { method: 'DELETE' });
  }

  async rotateSourceSecret(id: string): Promise<{ secret: string }> {
    return this.request<{ secret: string }>(`/api/v1/sources/${id}/rotate-secret`, {
      method: 'POST',
    });
  }

  async getSourceVerificationLog(id: string, limit = 20): Promise<VerificationLog[]> {
    return this.request<VerificationLog[]>(`/api/v1/sources/${id}/verification-log?limit=${limit}`);
  }

  // --- Destinations ---
  async listDestinations(): Promise<Destination[]> {
    return this.request<Destination[]>('/api/v1/destinations');
  }

  async createDestination(input: {
    name: string;
    url: string;
    description?: string;
    rate_limit?: number;
    timeout_ms?: number;
    max_retry_count?: number;
    secret?: string;
  }): Promise<Destination> {
    return this.request<Destination>('/api/v1/destinations', {
      method: 'POST',
      body: JSON.stringify(input),
    });
  }

  async updateDestination(id: string, input: Partial<Destination>): Promise<Destination> {
    return this.request<Destination>(`/api/v1/destinations/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(input),
    });
  }

  async deleteDestination(id: string): Promise<void> {
    return this.request<void>(`/api/v1/destinations/${id}`, { method: 'DELETE' });
  }

  async resetCircuit(id: string): Promise<void> {
    return this.request<void>(`/api/v1/destinations/${id}/circuit/reset`, { method: 'POST' });
  }

  // --- Subscriptions ---
  async listSubscriptions(): Promise<Subscription[]> {
    return this.request<Subscription[]>('/api/v1/subscriptions');
  }

  async createSubscription(input: {
    source_id: string;
    destination_id: string;
    event_types: string[];
    filter_expression?: string;
  }): Promise<Subscription> {
    return this.request<Subscription>('/api/v1/subscriptions', {
      method: 'POST',
      body: JSON.stringify(input),
    });
  }

  async updateSubscription(id: string, input: Partial<Subscription>): Promise<Subscription> {
    return this.request<Subscription>(`/api/v1/subscriptions/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(input),
    });
  }

  async deleteSubscription(id: string): Promise<void> {
    return this.request<void>(`/api/v1/subscriptions/${id}`, { method: 'DELETE' });
  }

  // --- Events ---
  async listEvents(limit = 50): Promise<EventItem[]> {
    return this.request<EventItem[]>(`/api/v1/events?limit=${limit}`);
  }

  async getEvent(id: string): Promise<EventItem> {
    return this.request<EventItem>(`/api/v1/events/${id}`);
  }

  async sendWebhook(slug: string, payload: any, headers?: Record<string, string>): Promise<any> {
    return this.request<any>(`/hooks/${slug}`, {
      method: 'POST',
      headers,
      body: JSON.stringify(payload),
    });
  }

  // --- Deliveries & DLQ ---
  async listDeliveries(params?: { status?: string; destination_id?: string; limit?: number }): Promise<Delivery[]> {
    const searchParams = new URLSearchParams();
    if (params?.status) searchParams.set('status', params.status);
    if (params?.destination_id) searchParams.set('destination_id', params.destination_id);
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    const q = searchParams.toString() ? `?${searchParams.toString()}` : '';
    return this.request<Delivery[]>(`/api/v1/deliveries${q}`);
  }

  async getDelivery(id: string): Promise<Delivery> {
    return this.request<Delivery>(`/api/v1/deliveries/${id}`);
  }

  async listDeliveryAttempts(id: string): Promise<DeliveryAttempt[]> {
    return this.request<DeliveryAttempt[]>(`/api/v1/deliveries/${id}/attempts`);
  }

  async replayDelivery(id: string): Promise<void> {
    return this.request<void>(`/api/v1/deliveries/${id}/replay`, { method: 'POST' });
  }

  async listDlq(limit = 50): Promise<{ items: DlqRecord[]; has_more: boolean }> {
    return this.request<{ items: DlqRecord[]; has_more: boolean }>(`/api/v1/dlq?limit=${limit}`);
  }

  async retryAllDlq(): Promise<{ success: boolean; requeued_count: number }> {
    return this.request<{ success: boolean; requeued_count: number }>('/api/v1/dlq/retry-all', {
      method: 'POST',
    });
  }

  async discardDlq(id: string): Promise<void> {
    return this.request<void>(`/api/v1/dlq/${id}`, { method: 'DELETE' });
  }

  // --- Transformations ---
  async listTransformations(subscriptionId?: string): Promise<Transformation[]> {
    const q = subscriptionId ? `?subscription_id=${subscriptionId}` : '';
    return this.request<Transformation[]>(`/api/v1/transformations${q}`);
  }

  async testTransformation(template: string, payload: any): Promise<{ transformed_payload: any }> {
    return this.request<{ transformed_payload: any }>('/api/v1/transformations/test', {
      method: 'POST',
      body: JSON.stringify({ template, payload }),
    });
  }

  // --- Stats ---
  async getOverviewStats(period = '24h'): Promise<OverviewStats> {
    return this.request<OverviewStats>(`/api/v1/stats/overview?period=${period}`);
  }

  async getTimeseries(metric = 'volume', period = '24h'): Promise<TimeseriesPoint[]> {
    return this.request<TimeseriesPoint[]>(`/api/v1/stats/timeseries?metric=${metric}&period=${period}`);
  }

  async getSystemStats(): Promise<SystemStats> {
    return this.request<SystemStats>('/api/v1/stats/system');
  }

  // --- API Keys ---
  async listApiKeys(): Promise<ApiKey[]> {
    return this.request<ApiKey[]>('/api/v1/api-keys');
  }

  async createApiKey(name: string, expiresInDays?: number): Promise<ApiKeyCreated> {
    return this.request<ApiKeyCreated>('/api/v1/api-keys', {
      method: 'POST',
      body: JSON.stringify({ name, expires_in_days: expiresInDays }),
    });
  }

  async revokeApiKey(id: string): Promise<void> {
    return this.request<void>(`/api/v1/api-keys/${id}`, { method: 'DELETE' });
  }
}

export const api = new ApiService();
