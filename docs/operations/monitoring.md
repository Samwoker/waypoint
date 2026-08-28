# Observability & Monitoring

RelayCore exposes deep health probes, Prometheus metrics, and structured logs to integrate with Datadog, Prometheus, Grafana, and ELK/Loki.

---

## 🩺 Health Probes

### 1. Kubernetes Liveness & Readiness Probe
`GET /healthz`

Returns HTTP 200 if both PostgreSQL and Redis are responsive:

```json
{
  "db": "ok",
  "queue": "ok"
}
```

If either component fails, returns HTTP 503 Service Unavailable with the failing subsystem status.

---

## 📊 Key Prometheus Metrics to Alert On

| Metric Name | Type | Recommended Alert Condition | Action Required |
| :--- | :--- | :--- | :--- |
| `relay_dead_letter_count` | Gauge | `> 0` for $> 5\text{m}$ | Investigate failed deliveries in the DLQ dashboard. |
| `relay_circuit_tripped_total` | Counter | Rate $> 0$ | A downstream destination is experiencing repeated failures. |
| `relay_delivery_duration_seconds{quantile="0.95"}` | Histogram | `> 3.0s` | Downstream receivers are responding slowly. |
| `relay_db_pool_active_connections` | Gauge | `> 90%` of max | Increase `DATABASE_MAX_CONNECTIONS` or optimize queries. |
