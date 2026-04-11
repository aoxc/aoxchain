# AOXChain RPC/API Reference (Current Surface)

This document captures the current repository HTTP/gRPC API surface for `aoxcrpc`.
It is intentionally implementation-aligned and should be updated when routes or method catalogs change.

## 1. HTTP Endpoints

Base form:

```text
http://<rpc-host>:<rpc-port>
```

For local operator smoke tests:

```bash
export AOXC_RPC_BASE="http://127.0.0.1:8545"
```

## 1.1 Global HTTP Security Controls (Implemented)

The HTTP API kernel currently enforces the following guards before route execution:

- Request-scoped rate limiting by client key.
- JSON `POST` content-type requirement (`application/json`).
- Maximum JSON body size (`RpcConfig.max_json_body_bytes`).
- mTLS client fingerprint enforcement on privileged contract mutation routes.
- Structured machine-readable error payloads including retry hints where applicable.

Operational hardening recommendation for `curl` clients:

- Always send `Accept: application/json`.
- Always send `Content-Type: application/json` on `POST` routes.
- Use explicit request timeouts (`--connect-timeout`, `--max-time`).
- Use retry policy only for safe/idempotent reads (`GET` routes), not privileged mutations.

### Privileged routes requiring mTLS

- `/contracts/register`
- `/contracts/activate`
- `/contracts/deprecate`
- `/contracts/revoke`

## 1.2 Health

- **Route:** `GET /health`
- **Purpose:** Node RPC health and readiness summary.

Example:

```bash
curl -sS \
  --connect-timeout 2 --max-time 10 \
  -H 'Accept: application/json' \
  "$AOXC_RPC_BASE/health" | jq .
```

## 1.3 Prometheus Metrics

- **Route:** `GET /metrics`
- **Purpose:** Prometheus exposition format for request counts, rejection counts, rate limiter state, and readiness score.

Example:

```bash
curl -sS \
  --connect-timeout 2 --max-time 10 \
  "$AOXC_RPC_BASE/metrics"
```

## 1.4 Cryptographic Profile Endpoints

- **Route:** `GET /quantum/profile`
- **Purpose:** Active quantum/crypto profile summary.

- **Route:** `GET /quantum/profile/full`
- **Purpose:** Extended cryptographic profile details for audit and operator diagnostics.

Example:

```bash
curl -sS \
  --connect-timeout 2 --max-time 10 \
  -H 'Accept: application/json' \
  "$AOXC_RPC_BASE/quantum/profile/full" | jq .
```

## 1.5 Contract Control Endpoints

All contract routes are JSON `POST` APIs:

- `/contracts/validate`
- `/contracts/register`
- `/contracts/get`
- `/contracts/list`
- `/contracts/activate`
- `/contracts/deprecate`
- `/contracts/revoke`
- `/contracts/runtime-binding`

Example (`list`):

```bash
curl -sS \
  --connect-timeout 2 --max-time 10 \
  -H 'Accept: application/json' \
  -H 'Content-Type: application/json' \
  -X POST "$AOXC_RPC_BASE/contracts/list" \
  -d '{"page":1,"page_size":20,"request_id":"req-1"}' | jq .
```

Privileged route example (`register`, mTLS required):

```bash
curl -sS \
  --connect-timeout 2 --max-time 10 \
  --cert client.crt --key client.key --cacert ca.crt \
  -H 'Accept: application/json' \
  -H 'Content-Type: application/json' \
  -X POST "$AOXC_RPC_BASE/contracts/register" \
  -d '{"request_id":"req-2","manifest":{}}' | jq .
```

## 1.6 `curl` Compatibility and Reliability Profile

The following command pattern is recommended for production-friendly read probes:

```bash
curl -sS --fail-with-body \
  --connect-timeout 2 --max-time 10 \
  --retry 2 --retry-delay 1 --retry-connrefused \
  -H 'Accept: application/json' \
  "$AOXC_RPC_BASE/health"
```

Notes:

- `--fail-with-body` preserves response payload while still surfacing non-2xx as failure.
- Retry configuration should be restricted to non-mutating routes.
- For privileged routes, enforce mTLS material loading from managed secret storage.

## 1.7 Full Query Surface Mapping (CLI + RPC)

The operator-facing query surface can be consumed both from CLI and HTTP probes.

| Diagnostic intent | CLI command | HTTP route |
| --- | --- | --- |
| RPC/API liveness | `aoxc api status` / `aoxc rpc-status` | `GET /health` |
| Runtime health | `aoxc api health` / `aoxc query runtime` | `GET /health` |
| Metrics snapshot | `aoxc api metrics` | `GET /metrics` |
| Crypto/quantum posture | `aoxc api contract` + `aoxc query vm status` (contextual) | `GET /quantum/profile`, `GET /quantum/profile/full` |
| Contract registry query | `aoxc api contract` (descriptor), `aoxc query vm contract` (state context) | `POST /contracts/get`, `POST /contracts/list` |
| Contract validation | CLI VM/query workflows | `POST /contracts/validate` |
| Privileged contract mutations | Operator-controlled workflows | `POST /contracts/register`, `/contracts/activate`, `/contracts/deprecate`, `/contracts/revoke` |

This mapping is intended to close operator ambiguity for “full query” diagnostics by making CLI and HTTP read paths explicit and auditable.

## 2. gRPC Method Catalog

The gRPC service currently exposes method catalog entries including:

- `query.GetChainStatus`
- `tx.Submit`

Use the crate’s server metadata/tests as source of truth for the catalog during upgrades.

## 3. Error Model

Unsupported routes return structured RPC errors (e.g., `METHOD_NOT_FOUND`).
Malformed JSON payloads return `INVALID_REQUEST` with a user hint to provide a valid JSON body matching the target schema.
Rate-limited requests return `RATE_LIMIT_EXCEEDED` with `retry_after_ms`.
Privileged mutation requests without valid mTLS identity return `MTLS_AUTH_FAILED`.

## 4. OpenAPI / Swagger Status

AOXChain currently documents RPC surfaces via implementation-aligned markdown and contract mappers.
A full OpenAPI artifact can be added in a follow-up once route schemas are stabilized for long-term compatibility support.
