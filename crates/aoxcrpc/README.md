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
