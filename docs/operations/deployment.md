# Production Deployment Guide

This guide describes deploying RelayCore in production environments using **Docker Compose** or **Kubernetes**.

---

## 🏗️ Production Topology

```mermaid
flowchart TB
    LB["Load Balancer / Ingress<br/>(TLS 1.3 Termination)"]
    
    subgraph API Cluster
        API1["API Instance 1"]
        API2["API Instance 2"]
    end

    subgraph Worker Cluster
        W1["Worker Pool Instance 1"]
        W2["Worker Pool Instance 2"]
    end

    subgraph State & Persistence
        PG[("PostgreSQL Cluster<br/>(Primary + Read Replica)")]
        Redis[("Redis Cluster / Sentinel<br/>(Queue & Delayed Tasks)")]
    end

    LB --> API1 & API2
    API1 & API2 --> PG & Redis
    Redis --> W1 & W2
    W1 & W2 --> PG
```

---

## 🐳 Option 1: Docker Compose Deployment

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    restart: always
    environment:
      POSTGRES_DB: webhook_relay
      POSTGRES_USER: relay_app
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - pgdata:/var/lib/postgresql/data
    networks:
      - internal

  redis:
    image: redis:7-alpine
    restart: always
    command: redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}
    volumes:
      - redisdata:/data
    networks:
      - internal

  api:
    image: ghcr.io/samwoker/waypoint-api:latest
    restart: always
    depends_on:
      - postgres
      - redis
    environment:
      DATABASE_URL: postgres://relay_app:${POSTGRES_PASSWORD}@postgres:5432/webhook_relay
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379
      API_HOST: 0.0.0.0
      API_PORT: 3001
      JWT_SECRET: ${JWT_SECRET}
      LOG_LEVEL: info
    ports:
      - "3001:3001"
    networks:
      - internal
      - public

  worker:
    image: ghcr.io/samwoker/waypoint-worker:latest
    restart: always
    depends_on:
      - postgres
      - redis
    environment:
      DATABASE_URL: postgres://relay_app:${POSTGRES_PASSWORD}@postgres:5432/webhook_relay
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379
      WORKER_CONCURRENCY: 20
      LOG_LEVEL: info
    networks:
      - internal

volumes:
  pgdata:
  redisdata:

networks:
  internal:
  public:
```

---

## ☸️ Option 2: Kubernetes Scaling Strategy

- **API Gateway (Stateless)**: Deploy with `HorizontalPodAutoscaler` (HPA) targeting CPU utilization $> 70\%$ or request latency $> 10\text{ms}$.
- **Background Worker (Task-based)**: Scale pods based on Redis pending queue depth using KEDA (Kubernetes Event-driven Autoscaling).
