# AOXChain Operator Surface Roadmap

## Scope

This document defines a production-oriented operator experience roadmap for AOXChain.
The objective is a deterministic and fail-closed control plane that remains accessible to
non-expert operators.

Design constraints:

- Deterministic outputs and repeatable workflows
- Auditable runtime, bootstrap, and lifecycle operations
- Fail-closed preflight and runtime checks
- Separation of concerns between CLI logic, Make wrappers, and shell orchestration

## Operator Experience Tiers

### Tier 1 — Single-command local chain

Goal: one command provisions a local chain and proves end-to-end operability.

Expected flow:

1. Create local workspace
2. Materialize genesis and validator metadata
3. Start local network processes
4. Seed faucet/account balances
5. Execute transfer smoke
6. Execute stake smoke
7. Print health and finality summary

Primary entrypoint:

- `make demo`

### Tier 2 — Guided devnet/testnet

Goal: explicit but guided workflow for environment-specific test operations.

Expected flow:

1. Genesis workspace initialization
2. Validator/account population
3. Genesis validation and fingerprinting
4. Network startup and smoke verification
5. Runtime doctor checks and exportable audit signals

Primary entrypoints:

- `make localnet`
- `make devnet`
- `make testnet`

### Tier 3 — Production-grade network control

Goal: controlled staged activation with deterministic policy verification.

Expected flow:

1. Offline genesis and manifest preparation
2. Signed artifact verification
3. Key and validator consistency checks
4. Staged network activation
5. Health/finality drift monitoring and evidence export

Primary entrypoints:

- `make doctor`
- `make audit-chain`
- `make network-start`
- `make network-stop`

## Command Surface Direction

The long-term user-facing command plane should evolve into grouped subcommands (for
example `aoxc chain`, `aoxc genesis`, `aoxc validator`, `aoxc wallet`, `aoxc tx`,
`aoxc stake`, `aoxc node`, `aoxc network`, `aoxc doctor`, `aoxc audit`).

Current repository state already contains foundational primitives (`genesis-*`,
`config-*`, `node-*`, runtime lifecycle and smoke checks). The roadmap is to layer:

1. Human-safe defaults and guided UX
2. Wallet/account/transfer/stake operator clarity
3. Explicit validator lifecycle controls
4. Multi-node orchestration and lifecycle controls

## Enforcement Rule

All critical validation must live in `aoxc` command implementations. Shell scripts are
orchestration-only wrappers and must not become the source of truth for safety-critical
checks.

## Added Wrapper Scripts

To align with this roadmap, the repository now provides orchestration helpers under
`scripts/`:

- `chain_demo.sh`
- `chain_create.sh`
- `network_start.sh`
- `network_stop.sh`
- `validator_bootstrap.sh`
- `wallet_seed.sh`
- `preflight_check.sh`
- `finality_smoke.sh`
- `transfer_smoke.sh`
- `runtime_recover.sh`

These wrappers are intentionally thin and defer operational logic to `make` targets
and `aoxc` subcommands.

## API and Control-Surface Completeness Matrix (Current Repository State)

This section records the implemented status of user-facing control surfaces so operators
can distinguish production-ready behavior from scaffolded or partial capabilities.

| Surface | Current status | What is implemented now | Gaps / non-final areas |
| --- | --- | --- | --- |
| HTTP RPC (curl-consumable) | **Full (repository scope)** | Deterministic route dispatch for health, metrics, quantum profile, and contract control-plane operations with structured JSON success/error envelopes. | Production deployment still requires environment-level TLS/key provisioning and network policy controls. |
| gRPC | **Full (repository scope)** | Startup checks, explicit method catalog, query and tx dispatch methods, and deterministic service-level request validation. | Production deployment still requires service host integration and release-profile hardening. |
| WebSocket RPC | **Full (repository scope)** | Session connect/disconnect, topic subscription, and deterministic fan-out payload delivery for block-confirmed events. | Production deployment still requires external transport host integration and runtime capacity controls. |
| AOXC CLI | **Full operator plane** | Command routing spans chain/genesis/validator/wallet/account/node/network/tx/stake/doctor/audit and compatibility aliases. | Operator runbooks still need environment-specific release evidence before each public rollout. |
| Chain query ergonomics | **Full baseline surface** | CLI query commands and typed query service admission-aware responses are implemented and mapped across operator and API layers. | Environment parity validation should still be executed in pre-release and production gates. |

### Operator interpretation rule

The repository now implements complete control-plane primitives for HTTP, gRPC, WebSocket,
CLI, and baseline query surfaces. Production go-live remains conditional on environment
readiness evidence, key-management posture, and release gate compliance.

## CLI / API / curl Compatibility Baseline

The following mapping is the canonical interoperability baseline for operator workflows.

| Capability | CLI surface | HTTP (curl) surface | gRPC surface |
| --- | --- | --- | --- |
| Chain health/readiness | `aoxc rpc-status` / `aoxc chain-status` | `GET /health` | `query.GetChainStatus` |
| Chain status query | `aoxc chain-status` | `POST /v1/query/chain-status` | `query.GetChainStatus` |
| Transaction submit | `aoxc tx ...` workflows | `POST /v1/tx/submit` | `tx.Submit` |
| Runtime security profile | `aoxc rpc-status` + diagnostics | `GET /quantum/profile` | N/A (HTTP profile endpoint) |
| Metrics export | `aoxc metrics` | `GET /metrics` | N/A (Prometheus exposition) |

### curl reference commands

```bash
curl -sS http://127.0.0.1:8080/health
curl -sS http://127.0.0.1:8080/metrics
curl -sS http://127.0.0.1:8080/quantum/profile
curl -sS -X POST http://127.0.0.1:8080/v1/query/chain-status \
  -H 'content-type: application/json' \
  -d '{"height":42,"syncing":false}'
curl -sS -X POST http://127.0.0.1:8080/v1/tx/submit \
  -H 'content-type: application/json' \
  -d '{
    "actor_id":"actor-1",
    "tx_payload":[1,2,3,4],
    "zkp_proof":[9,9,9,9,9,9,9,9],
    "identity_tier":"signed_client",
    "signer_algorithms":["ed25519","ml-dsa-65"],
    "remaining_budget_units":100
  }'
```
