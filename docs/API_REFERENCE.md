# AOXChain RPC/API Reference (Current Surface)

This document captures the current repository HTTP/gRPC API surface for `aoxcrpc`.
It is intentionally implementation-aligned and should be updated when routes or method catalogs change.

## 1. HTTP Endpoints

Base form:

```text
http://<rpc-host>:<rpc-port>
```

### 1.1 Health

- **Route:** `GET /health`
- **Purpose:** Node RPC health and readiness summary.

Example:

```bash
curl -s http://127.0.0.1:8545/health | jq .
```

### 1.2 Prometheus Metrics

- **Route:** `GET /metrics`
- **Purpose:** Prometheus exposition format for request counts, rejection counts, rate limiter state, and readiness score.

Example:

```bash
curl -s http://127.0.0.1:8545/metrics
```

### 1.3 Cryptographic Profile Endpoints

- **Route:** `GET /quantum/profile`
- **Purpose:** Active quantum/crypto profile summary.

- **Route:** `GET /quantum/profile/full`
- **Purpose:** Extended cryptographic profile details for audit and operator diagnostics.

Example:

```bash
curl -s http://127.0.0.1:8545/quantum/profile/full | jq .
```

### 1.4 Contract Control Endpoints

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
curl -s \
  -H 'Content-Type: application/json' \
  -X POST http://127.0.0.1:8545/contracts/list \
  -d '{"page":1,"limit":20}' | jq .
```

## 2. gRPC Method Catalog

The gRPC service currently exposes method catalog entries including:

- `query.GetChainStatus`
- `tx.Submit`

Use the crate’s server metadata/tests as source of truth for the catalog during upgrades.

## 3. Error Model

Unsupported routes return structured RPC errors (e.g., `METHOD_NOT_FOUND`).
Malformed JSON payloads return `INVALID_REQUEST` with a user hint to provide a valid JSON body matching the target schema.

## 4. OpenAPI / Swagger Status

AOXChain currently documents RPC surfaces via implementation-aligned markdown and contract mappers.
A full OpenAPI artifact can be added in a follow-up once route schemas are stabilized for long-term compatibility support.
