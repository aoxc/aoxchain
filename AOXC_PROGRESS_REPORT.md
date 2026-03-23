# AOXC Progress Report

- Profile: `validator`
- Stage: `integration-hardening`
- Overall readiness: **60%** (73/121)
- Verdict: `not-ready`

## Dual-track progress

- **testnet**: 65% (73/111) — in-progress
  - Objective: Public testnet should close all non-mainnet-specific blockers and sustain AOXHub/core parity.
- **mainnet**: 60% (73/121) — in-progress
  - Objective: Mainnet requires every weighted control to pass, including production profile, keys, runtime, and release evidence.

## Area progress

- **configuration**: 60% (1/2 checks, weight 15/25) — in-progress
- **network**: 100% (1/1 checks, weight 10/10) — ready
- **observability**: 50% (1/2 checks, weight 8/16) — in-progress
- **identity**: 0% (0/2 checks, weight 0/22) — bootstrap
- **runtime**: 0% (0/1 checks, weight 0/8) — bootstrap
- **release**: 100% (6/6 checks, weight 29/29) — ready
- **operations**: 100% (2/2 checks, weight 11/11) — ready

## Remaining blockers

- mainnet-profile: Active profile is validator
- structured-logging: JSON logs are required for audit trails and SIEM ingestion
- genesis-present: Committed genesis material must exist in AOXC home
- node-state-present: Node runtime state must exist and load cleanly
- operator-key-active: Operator key operational state is missing

## Recommended next focus

- identity: raise from 0% to 100% (0 of 2 checks passing)
- runtime: raise from 0% to 100% (0 of 1 checks passing)
- observability: raise from 50% to 100% (1 of 2 checks passing)

## Remediation plan

- Run `aoxc production-bootstrap --profile mainnet --password <value>` or `aoxc config-init --profile mainnet --json-logs`.
- Enable JSON logging to preserve audit-quality operator trails and SIEM ingestion.
- Materialize genesis with `aoxc genesis-init` or re-run `aoxc production-bootstrap`.
- Initialize runtime state with `aoxc node-bootstrap` or re-run `aoxc production-bootstrap`.
- Bootstrap or rotate operator keys with `aoxc key-bootstrap --profile mainnet --password <value>`.

## Baseline parity

### Embedded network profiles

- Status: **aligned**
- Mainnet file: `/workspace/aoxchain/configs/mainnet.toml`
- Testnet file: `/workspace/aoxchain/configs/testnet.toml`
- security_mode: ok (mainnet=`audit_strict`, testnet=`audit_strict`)
- peer_seed_count: ok (mainnet=`2`, testnet=`2`)
- listen_port_offset: ok (mainnet=`26656/8545`, testnet=`36656/18545`)

### AOXHub network profiles

- Status: **aligned**
- Mainnet file: `/workspace/aoxchain/configs/aoxhub-mainnet.toml`
- Testnet file: `/workspace/aoxchain/configs/aoxhub-testnet.toml`
- security_mode: ok (mainnet=`audit_strict`, testnet=`audit_strict`)
- peer_seed_count: ok (mainnet=`2`, testnet=`2`)
- listen_port_offset: ok (mainnet=`27656/9545`, testnet=`37656/19545`)


## Check matrix

- [PASS] **config-valid** / configuration / weight 15 — Operator configuration passed validation
- [FAIL] **mainnet-profile** / configuration / weight 10 — Active profile is validator
- [PASS] **official-peers** / network / weight 10 — Official peer enforcement must remain enabled for production
- [PASS] **telemetry-metrics** / observability / weight 8 — Prometheus/metrics export is required for production operations
- [FAIL] **structured-logging** / observability / weight 8 — JSON logs are required for audit trails and SIEM ingestion
- [FAIL] **genesis-present** / identity / weight 10 — Committed genesis material must exist in AOXC home
- [FAIL] **node-state-present** / runtime / weight 8 — Node runtime state must exist and load cleanly
- [FAIL] **operator-key-active** / identity / weight 12 — Operator key operational state is missing
- [PASS] **profile-baseline-parity** / release / weight 8 — Mainnet and testnet embedded baselines share the same production control shape
- [PASS] **aoxhub-baseline-parity** / release / weight 5 — AOXHub mainnet/testnet baselines are aligned with the same security and port model
- [PASS] **release-evidence** / release / weight 7 — Release evidence bundle must exist under /workspace/aoxchain/artifacts/release-evidence
- [PASS] **production-closure** / operations / weight 7 — Production closure artifacts must exist under /workspace/aoxchain/artifacts/network-production-closure
- [PASS] **security-drill-evidence** / operations / weight 4 — Security drill evidence must capture penetration, RPC hardening, and session replay scenarios
- [PASS] **desktop-wallet-hub-compat** / release / weight 4 — Desktop wallet compatibility evidence must cover AOXHub plus mainnet/testnet routing
- [PASS] **compatibility-matrix** / release / weight 3 — Compatibility matrix evidence must be generated for the candidate release
- [PASS] **provenance-attestation** / release / weight 2 — Provenance attestation must exist before final mainnet sign-off
