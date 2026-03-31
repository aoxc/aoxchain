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
