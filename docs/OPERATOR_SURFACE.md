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


## Deterministic Bootstrap Breakdown

This section decomposes bootstrap into explicit phases so operators can run, audit,
and recover the process without hidden assumptions.

### Bootstrap Phase 0 — Environment and artifact boundary

Purpose:

- Establish the exact environment profile (`localnet`, `devnet`, `testnet`,
  `validation`, or `mainnet`)
- Freeze artifact inputs before node activation

Required inputs:

- `profile.toml`
- `manifest.v1.json`
- `release-policy.toml`
- `certificate.json`

Expected controls:

1. Verify all required files exist and are readable.
2. Validate manifest identity fields are internally consistent.
3. Verify certificate and release policy compatibility with target profile.
4. Record immutable fingerprints (for example SHA-256 digests) in operator logs.

Fail-closed behavior:

- If any required artifact is missing or malformed, bootstrap stops before key
  loading or process startup.

### Bootstrap Phase 1 — Genesis and topology integrity

Purpose:

- Guarantee deterministic chain identity and peer layout before any runtime action

Required inputs:

- `genesis.v1.json`
- `validators.json`
- `bootnodes.json`
- Topology policy files under `topology/`

Expected controls:

1. Validate genesis schema and chain/network identifiers.
2. Validate validator set structure, uniqueness, and identity linkage.
3. Validate bootnode records and endpoint formatting.
4. Validate topology matrix and role mapping constraints.
5. Confirm hash compatibility between genesis and manifest references.

Fail-closed behavior:

- Any mismatch across genesis/validator/bootnode/topology surfaces aborts
  bootstrap and prevents partial startup.

### Bootstrap Phase 2 — Node identity and local trust material

Purpose:

- Confirm each node starts with deterministic and non-conflicting local identity

Required inputs:

- Node identity material in environment-specific node homes
- Local host mapping and socket matrix policy

Expected controls:

1. Verify node home layout and permissions.
2. Verify seed/key files are present for all required nodes.
3. Validate local endpoint uniqueness (no port collisions in role topology).
4. Verify node-role assignments match consensus policy expectations.

Fail-closed behavior:

- Missing identity artifacts, permission violations, or endpoint collisions block
  process launch.

### Bootstrap Phase 3 — Controlled process activation

Purpose:

- Activate processes in a deterministic order with immediate health feedback

Expected controls:

1. Start required bootstrap node set first.
2. Start remaining validator/observer nodes according to role topology.
3. Perform bounded readiness checks after each start stage.
4. Abort and roll back startup if critical readiness thresholds are not met.

Fail-closed behavior:

- Startup halts when required quorum or core service readiness is not achieved
  within policy-defined limits.

### Bootstrap Phase 4 — Post-start verification and smoke signals

Purpose:

- Prove network operability immediately after activation

Expected controls:

1. Run chain/rpc health checks.
2. Run finality smoke checks.
3. Run transfer smoke checks where required by environment policy.
4. Export deterministic status and evidence artifacts.

Fail-closed behavior:

- A network that starts but fails post-start smoke checks is treated as not
  bootstrap-complete.

### Bootstrap Phase 5 — Audit closure and handoff

Purpose:

- Create a reviewable closure package for operations and governance consumers

Expected controls:

1. Export fingerprints, health summaries, and smoke outcomes.
2. Record bootstrap timestamp window and operator command trace.
3. Persist closure artifacts under `artifacts/` or the environment evidence path.
4. Mark bootstrap state as complete only after evidence export succeeds.

Fail-closed behavior:

- Bootstrap success is not declared until audit closure artifacts are durable and
  reviewable.

### Minimal acceptance definition for "bootstrap complete"

Bootstrap is complete only when all of the following are true:

1. Artifact validation passed (Phase 0 and Phase 1).
2. Node identity and topology checks passed (Phase 2).
3. Deterministic startup reached required readiness thresholds (Phase 3).
4. Health/finality/smoke checks passed (Phase 4).
5. Audit closure artifacts were exported and persisted (Phase 5).

Any missing condition must be reported as "bootstrap incomplete" in operator output.

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
| HTTP RPC (curl-consumable) | **Partial** | Health, metrics snapshot, quantum profile payload builders, and contract HTTP-style request/response handlers with validation and mapping logic. Operator CLI also exposes `rpc-status` and `rpc-curl-smoke` probes for endpoint and JSON-RPC status verification against the configured listener. | No end-to-end HTTP router/listener wiring is defined in this document set; operators should treat this as API-domain implementation, not confirmed full deployment surface. |
| gRPC | **Partial** | Server object and startup checks, query/tx service structs, admission checks, and deterministic response modeling. | Transport serving loop, full method catalog exposure, and network deployment hardening are not yet represented as complete runtime wiring here. |
| WebSocket RPC | **Partial** | Event struct and deterministic JSON event formatting for confirmed-block notifications. | Subscription lifecycle, connection/session management, and streaming orchestration are not represented as complete production flow. |
| AOXC CLI | **Broad but evolving** | Command routing spans chain/genesis/validator/wallet/account/node/network/query/tx/stake/doctor/audit and many compatibility aliases. | Command breadth is high, but not every command implies finalized mainnet-grade semantics; compatibility and behavior should be validated per command path. |
| Chain query ergonomics | **Defined but not exhaustive** | Flat query commands remain available, and grouped query routing now supports `aoxc query chain|consensus|vm|network|block|tx|receipt|account|balance|state-root|rpc` for a canonical read-path UX. | Unified query contract/versioning and external RPC query parity still require explicit operator validation in each target environment. |

### Operator interpretation rule

Until full endpoint wiring and environment-level readiness evidence are published for each
transport surface, treat HTTP/gRPC/WebSocket as **implemented modules with partial
end-to-end closure**, not as an unconditional “fully complete” public API guarantee.
