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
| HTTP RPC (curl-consumable) | **Partial** | Health, metrics snapshot, quantum profile payload builders, and contract HTTP-style request/response handlers with validation and mapping logic. | No end-to-end HTTP router/listener wiring is defined in this document set; operators should treat this as API-domain implementation, not confirmed full deployment surface. |
| gRPC | **Partial** | Server object and startup checks, query/tx service structs, admission checks, and deterministic response modeling. | Transport serving loop, full method catalog exposure, and network deployment hardening are not yet represented as complete runtime wiring here. |
| WebSocket RPC | **Partial** | Event struct and deterministic JSON event formatting for confirmed-block notifications. | Subscription lifecycle, connection/session management, and streaming orchestration are not represented as complete production flow. |
| AOXC CLI | **Broad but evolving** | Command routing spans chain/genesis/validator/wallet/account/node/network/query/tx/stake/doctor/audit and many compatibility aliases. | Command breadth is high, but not every command implies finalized mainnet-grade semantics; compatibility and behavior should be validated per command path. |
| Chain query ergonomics | **Defined but not exhaustive** | Flat query commands remain available; grouped query routing supports `aoxc query chain|consensus|vm|network|block|tx|receipt|account|balance|state-root|rpc`, including consensus detail paths (`validators`, `proposer`, `round`, `finality`, `commits`, `evidence`) and VM read/simulate helpers (`call`, `simulate`, `storage`, `contract`, `code`, `estimate-gas`, `trace`). | Unified query contract/versioning and external RPC query parity still require explicit operator validation in each target environment. |

### Operator interpretation rule

Until full endpoint wiring and environment-level readiness evidence are published for each
transport surface, treat HTTP/gRPC/WebSocket as **implemented modules with partial
end-to-end closure**, not as an unconditional “fully complete” public API guarantee.
