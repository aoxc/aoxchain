# aoxcrpc

## Purpose

`aoxcrpc` is responsible for the **API ingress layer (HTTP/gRPC/WebSocket)** domain within the AOXChain workspace.

## Code Scope

- `proto/`
- `src/middleware/`
- `src/grpc/`
- `src/http/`
- `src/websocket/`
- `src/config.rs`

## Operational Notes
This crate now includes a production-oriented secure API skeleton:

- `proto/` definitions for binary gRPC contracts,
- security middleware (`mTLS`, `rate limiting`, `ZKP validation`),
- split service boundaries for query and transaction submission,
- HTTP health + Prometheus metrics snapshot export,
- websocket event framing for block confirmations.
- rate limiter rejections include `retry_after_ms` metadata for deterministic client backoff UX.
- canonical `RpcErrorResponse` model is available for machine-readable error payloads (`code`, `message`, `retry_after_ms`, `request_id`).
- in-memory limiter supports stale key pruning and bounded key tracking (LRU-style eviction) to control long-run memory growth risk.
- Prometheus snapshot includes `aox_rpc_rate_limited_total` and `aox_rpc_rate_limiter_active_keys` for abuse-visibility.
- rate limiter rejections now include `retry_after_ms` metadata to support client-side backoff UX.
- in-memory limiter supports stale key pruning to prevent unbounded map growth in long-running nodes.

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxcrpc
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
