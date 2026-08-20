# Multi-stage Dockerfile for Webhook Relay (api and worker)

# 1. Builder Stage
FROM rust:1.80-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static git

# Copy workspace source files
COPY Cargo.toml Cargo.lock* ./
COPY crates ./crates
COPY migrations ./migrations

# Build release binaries for the workspace
RUN cargo build --release --workspace

# 2. Runtime Target: API
FROM alpine:3.20 AS api

WORKDIR /app
RUN apk add --no-cache ca-certificates libssl3

COPY --from=builder /app/target/release/api /usr/local/bin/api
COPY migrations ./migrations

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/api"]

# 3. Runtime Target: Worker
FROM alpine:3.20 AS worker

WORKDIR /app
RUN apk add --no-cache ca-certificates libssl3

COPY --from=builder /app/target/release/worker /usr/local/bin/worker
COPY migrations ./migrations

ENTRYPOINT ["/usr/local/bin/worker"]
