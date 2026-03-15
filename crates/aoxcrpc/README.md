# aoxcrpc

## Purpose

RPC ingress surfaces (HTTP, gRPC, WebSocket) for operator and integration-facing APIs.

## Production Intent

This crate now includes a production-oriented secure API skeleton:

- `proto/` definitions for binary gRPC contracts,
- security middleware (`mTLS`, `rate limiting`, `ZKP validation`),
- split service boundaries for query and transaction submission,
- HTTP health + Prometheus metrics snapshot export,
- websocket event framing for block confirmations.

## Local Development

From repository root:

```bash
cargo check -p aoxcrpc
```

## Integration Notes

- Keep API changes synchronized with dependent crates in the same pull request.
- For consensus/network/identity touching changes, include tests or deterministic command paths.
- Avoid introducing implicit defaults in critical runtime logic; prefer explicit parameters.
