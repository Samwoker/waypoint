import {
  ApiKey,
  ApiKeyCreated,
  Delivery,
  DeliveryAttempt,
  DeliveryDetail,
  Destination,
  DestinationHealth,
  DlqRecord,
  EventDetail,
  EventDeliveryItem,
  EventItem,
  OverviewStats,
  PaginatedDeliveries,
  PaginatedDlq,
  PaginatedEvents,
  RawEventPayload,
  RotateSecretResponse,
  Source,
  Subscription,
  SystemStats,
  Tenant,
  TenantUsage,
  TestDestinationResponse,
  TimeseriesPoint,
  Transformation,
  User,
  VerificationLog,
} from '../types';

class ApiClient {
  private baseUrl: string = '';
  private token: string | null = localStorage.getItem('waypoint_token');

  constructor() {
    this.token = localStorage.getItem('waypoint_token');
  }

  setToken(token: string | null) {
    this.token = token;
    if (token) {
      localStorage.setItem('waypoint_token', token);
    } else {
      localStorage.removeItem('waypoint_token');
    }
  }

  getToken(): string | null {
    return this.token || localStorage.getItem('waypoint_token');
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string>),
    };

    const currentToken = this.getToken();
    if (currentToken && !headers['Authorization']) {
      headers['Authorization'] = `Bearer ${currentToken}`;
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      if (response.status === 401) {
        // Clear invalid token
        this.setToken(null);
      }
      let errMsg = `HTTP Error ${response.status}: ${response.statusText}`;
      try {
        const errorData = await response.json();
        errMsg = errorData.message || errorData.error || errMsg;
      } catch (_) {
        // ignore parse error
      }
      throw new Error(errMsg);
    }

    if (response.status === 204) {
      return {} as T;
    }

    return response.json();
  }

  // --- Auth & User ---
  async register(
    email: string,
    pass: string,
    tenantName: string
  ): Promise<{ access_token: string; refresh_token: string }> {
    const res = await this.request<{ access_token: string; refresh_token: string }>(
      '/api/v1/auth/register',
      {
        method: 'POST',
        body: JSON.stringify({ email, password: pass, tenant_name: tenantName }),
      }
    );
    this.setToken(res.access_token);
    return res;
  }

  async login(
    email: string,
    pass: string
  ): Promise<{ access_token: string; refresh_token: string }> {
    const res = await this.request<{ access_token: string; refresh_token: string }>(
      '/api/v1/auth/login',
      {
        method: 'POST',
        body: JSON.stringify({ email, password: pass }),
      }
    );
    this.setToken(res.access_token);
    return res;
  }

  async getMe(): Promise<User> {
    const res = await this.request<any>('/api/v1/auth/me');
    return {
      id: res.tenant_id || res.id,
      tenant_id: res.tenant_id,
      email: res.tenant?.name ? `${res.tenant.slug}@waypoint.internal` : 'admin@waypoint.internal',
      role: res.is_admin ? 'admin' : 'member',
      is_admin: !!res.is_admin,
      created_at: res.tenant?.created_at || new Date().toISOString(),
    };
  }

  // --- Sources ---
  async listSources(): Promise<Source[]> {
    return this.request<Source[]>('/api/v1/sources');
  }

  async getSource(id: string): Promise<Source> {
    return this.request<Source>(`/api/v1/sources/${id}`);
  }

  async createSource(data: {
    name: string;
    slug: string;
    description?: string;
    provider?: string;
    verification_type?: string;
    secret?: string;
  }): Promise<Source> {
    return this.request<Source>('/api/v1/sources', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateSource(id: string, data: Partial<Source>): Promise<Source> {
    return this.request<Source>(`/api/v1/sources/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deleteSource(id: string): Promise<void> {
    return this.request<void>(`/api/v1/sources/${id}`, { method: 'DELETE' });
  }

  async rotateSourceSecret(id: string): Promise<RotateSecretResponse> {
    return this.request<RotateSecretResponse>(`/api/v1/sources/${id}/rotate-secret`, {
      method: 'POST',
    });
  }

  async getSourceVerificationLog(id: string, limit: number = 20): Promise<VerificationLog[]> {
    return this.request<VerificationLog[]>(`/api/v1/sources/${id}/verification-log?limit=${limit}`);
  }

  // --- Destinations ---
  async listDestinations(): Promise<Destination[]> {
    return this.request<Destination[]>('/api/v1/destinations');
  }

  async getDestination(id: string): Promise<Destination> {
    return this.request<Destination>(`/api/v1/destinations/${id}`);
  }

  async createDestination(data: {
    name: string;
    url: string;
    description?: string;
    timeout_ms?: number;
    max_retries?: number;
    rate_limit_rps?: number;
  }): Promise<Destination> {
    return this.request<Destination>('/api/v1/destinations', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateDestination(id: string, data: Partial<Destination>): Promise<Destination> {
    return this.request<Destination>(`/api/v1/destinations/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deleteDestination(id: string): Promise<void> {
    return this.request<void>(`/api/v1/destinations/${id}`, { method: 'DELETE' });
  }

  async pauseDestination(id: string): Promise<Destination> {
    return this.request<Destination>(`/api/v1/destinations/${id}/pause`, { method: 'POST' });
  }

  async resumeDestination(id: string): Promise<Destination> {
    return this.request<Destination>(`/api/v1/destinations/${id}/resume`, { method: 'POST' });
  }

  async testDestination(id: string): Promise<TestDestinationResponse> {
    return this.request<TestDestinationResponse>(`/api/v1/destinations/${id}/test`, {
      method: 'POST',
    });
  }

  async getDestinationHealth(id: string): Promise<DestinationHealth> {
    return this.request<DestinationHealth>(`/api/v1/destinations/${id}/health`);
  }

  async getDestinationStats(id: string): Promise<any> {
    return this.request<any>(`/api/v1/stats/destinations/${id}`);
  }

  // --- Subscriptions ---
  async listSubscriptions(): Promise<Subscription[]> {
    return this.request<Subscription[]>('/api/v1/subscriptions');
  }

  async getSubscription(id: string): Promise<Subscription> {
    return this.request<Subscription>(`/api/v1/subscriptions/${id}`);
  }

  async createSubscription(data: {
    source_id: string;
    destination_id: string;
    event_types: string[];
    filter_rules?: any;
    transformation_template?: string;
  }): Promise<Subscription> {
    return this.request<Subscription>('/api/v1/subscriptions', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateSubscription(id: string, data: Partial<Subscription>): Promise<Subscription> {
    return this.request<Subscription>(`/api/v1/subscriptions/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deleteSubscription(id: string): Promise<void> {
    return this.request<void>(`/api/v1/subscriptions/${id}`, { method: 'DELETE' });
  }

  async pauseSubscription(id: string): Promise<Subscription> {
    return this.request<Subscription>(`/api/v1/subscriptions/${id}/pause`, { method: 'POST' });
  }

  async resumeSubscription(id: string): Promise<Subscription> {
    return this.request<Subscription>(`/api/v1/subscriptions/${id}/resume`, { method: 'POST' });
  }

  // --- Events ---
  async listEvents(limit: number = 20, cursor?: string): Promise<PaginatedEvents> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (cursor) params.append('cursor', cursor);
    return this.request<PaginatedEvents>(`/api/v1/events?${params.toString()}`);
  }

  async getEvent(id: string): Promise<EventDetail> {
    return this.request<EventDetail>(`/api/v1/events/${id}`);
  }

  async getEventRaw(id: string): Promise<RawEventPayload> {
    return this.request<RawEventPayload>(`/api/v1/events/${id}/raw`);
  }

  async getEventDeliveries(id: string): Promise<EventDeliveryItem[]> {
    return this.request<EventDeliveryItem[]>(`/api/v1/events/${id}/deliveries`);
  }

  async replayEvent(id: string): Promise<{ event_id: string; deliveries_created: number }> {
    return this.request<{ event_id: string; deliveries_created: number }>(
      `/api/v1/events/${id}/replay`,
      { method: 'POST' }
    );
  }

  async sendTestWebhook(slug: string, payload: any, eventType: string): Promise<any> {
    return this.request<any>(`/hooks/${slug}`, {
      method: 'POST',
      headers: {
        'X-Event-Type': eventType,
      },
      body: typeof payload === 'string' ? payload : JSON.stringify(payload),
    });
  }

  // --- Deliveries ---
  async listDeliveries(status?: string, limit: number = 20, cursor?: string): Promise<PaginatedDeliveries> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (status) params.append('status', status);
    if (cursor) params.append('cursor', cursor);
    return this.request<PaginatedDeliveries>(`/api/v1/deliveries?${params.toString()}`);
  }

  async getDelivery(id: string): Promise<DeliveryDetail> {
    return this.request<DeliveryDetail>(`/api/v1/deliveries/${id}`);
  }

  async getDeliveryAttempts(id: string): Promise<DeliveryAttempt[]> {
    return this.request<DeliveryAttempt[]>(`/api/v1/deliveries/${id}/attempts`);
  }

  async replayDelivery(id: string): Promise<void> {
    return this.request<void>(`/api/v1/deliveries/${id}/replay`, { method: 'POST' });
  }

  // --- Dead Letter Queue (DLQ) ---
  async listDlq(limit: number = 20, cursor?: string): Promise<PaginatedDlq> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (cursor) params.append('cursor', cursor);
    return this.request<PaginatedDlq>(`/api/v1/dlq?${params.toString()}`);
  }

  async requeueDlqItem(id: string): Promise<void> {
    return this.request<void>(`/api/v1/dlq/${id}/requeue`, { method: 'POST' });
  }

  async discardDlqItem(id: string): Promise<void> {
    return this.request<void>(`/api/v1/dlq/${id}`, { method: 'DELETE' });
  }

  async retryAllDlq(): Promise<{ replayed_count: number }> {
    return this.request<{ replayed_count: number }>('/api/v1/dlq/retry-all', { method: 'POST' });
  }

  // --- Transformations ---
  async listTransformations(): Promise<Transformation[]> {
    return this.request<Transformation[]>('/api/v1/transformations');
  }

  async testTransformation(template: string, payload: any): Promise<any> {
    return this.request<any>('/api/v1/transformations/test', {
      method: 'POST',
      body: JSON.stringify({ template, payload }),
    });
  }

  // --- API Keys ---
  async listApiKeys(): Promise<ApiKey[]> {
    return this.request<ApiKey[]>('/api/v1/api-keys');
  }

  async createApiKey(name: string, expiresAt?: string): Promise<ApiKeyCreated> {
    return this.request<ApiKeyCreated>('/api/v1/api-keys', {
      method: 'POST',
      body: JSON.stringify({ name, expires_at: expiresAt }),
    });
  }

  async revokeApiKey(id: string): Promise<void> {
    return this.request<void>(`/api/v1/api-keys/${id}`, { method: 'DELETE' });
  }

  // --- Statistics & Observability ---
  async getOverviewStats(period: string = '24h'): Promise<OverviewStats> {
    return this.request<OverviewStats>(`/api/v1/stats/overview?period=${period}`);
  }

  async getTimeseriesStats(period: string = '24h'): Promise<TimeseriesPoint[]> {
    return this.request<TimeseriesPoint[]>(`/api/v1/stats/timeseries?period=${period}`);
  }

  setApiKey(key: string | null) {
    if (key) {
      localStorage.setItem('waypoint_api_key', key);
    } else {
      localStorage.removeItem('waypoint_api_key');
    }
  }

  async getSystemStats(): Promise<SystemStats> {
    return this.request<SystemStats>('/api/v1/stats/system');
  }

  // --- Tenants & Usage ---
  async listTenants(): Promise<Tenant[]> {
    return this.request<Tenant[]>('/api/v1/tenants');
  }

  async getTenantUsage(tenantId: string, period: string = '30d'): Promise<TenantUsage> {
    return this.request<TenantUsage>(`/api/v1/tenants/${tenantId}/usage?period=${period}`);
  }
}

export const api = new ApiClient();
