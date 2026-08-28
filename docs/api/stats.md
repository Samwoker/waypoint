# Statistics & Observability API

The Statistics API exposes real-time telemetry, historical throughput timeseries, per-source/destination performance metrics, and Prometheus-compatible metrics.

---

## 1. Get Overview Telemetry Stats

`GET /api/v1/stats/overview?period=24h`

Returns aggregated key performance indicators for the tenant over the requested time window (`1h`, `24h`, `7d`, `30d`).

### Success Response (`200 OK`):
```json
{
  "total_events": 18450,
  "total_deliveries": 36900,
  "successful_deliveries": 36720,
  "failed_deliveries": 180,
  "success_rate": 0.9951,
  "p50_latency_ms": 42.0,
  "p95_latency_ms": 128.5,
  "dead_letter_count": 12,
  "period": "24h"
}
```

---

## 2. Get Throughput Timeseries Data

`GET /api/v1/stats/timeseries?period=24h`

Returns chronological bucketed event volume data points suitable for area/bar charts.

### Success Response (`200 OK`):
```json
[
  { "bucket": "2026-08-28 10:00", "value": 1520 },
  { "bucket": "2026-08-28 11:00", "value": 1780 },
  { "bucket": "2026-08-28 12:00", "value": 2100 }
]
```

---

## 3. Get System Stats (Platform Admin)

`GET /api/v1/stats/system`

Provides platform-wide operational statistics including active worker pools, queue depth, and memory usage.

### Success Response (`200 OK`):
```json
{
  "total_tenants": 48,
  "active_workers": 10,
  "pending_queue_depth": 34,
  "redis_memory_used_bytes": 14589000,
  "db_pool_active_connections": 8,
  "uptime_seconds": 864000
}
```

---

## 4. Prometheus Metrics Endpoint

`GET /metrics` *(Unauthenticated / Configured for Prometheus scraper)*

Emits standard Prometheus metrics for Grafana/Datadog scraping:

```text
# HELP relay_events_received_total Total number of webhook events received
# TYPE relay_events_received_total counter
relay_events_received_total{provider="stripe"} 14250
relay_events_received_total{provider="github"} 4200

# HELP relay_deliveries_total Total outbound delivery attempts
# TYPE relay_deliveries_total counter
relay_deliveries_total{status="delivered"} 36720
relay_deliveries_total{status="failed"} 180

# HELP relay_delivery_duration_seconds Latency of outbound delivery attempts
# TYPE relay_delivery_duration_seconds histogram
relay_delivery_duration_seconds_bucket{le="0.05"} 18450
relay_delivery_duration_seconds_bucket{le="0.1"} 32000
relay_delivery_duration_seconds_bucket{le="0.5"} 36500
```
