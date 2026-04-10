# AOXC-Q Consensus Specification (v0.2.0 Baseline)

## Purpose

This document defines the AOXC-Q consensus extension baseline used for the `AOXC-Q-v0.2.0` release line.

The engineering target is deterministic consensus continuity with crypto-agile controls, not unverifiable claims of absolute security.

## AOXC-Q Design Goals

1. Preserve deterministic state transition behavior.
2. Enforce fail-closed cryptographic profile admission.
3. Introduce role-segmented node topology for resilience and operational clarity.
4. Keep rollout reversible through explicit rollback controls.

## Node Roles

AOXC-Q deployment uses five operational node roles:

- **Q-Validator:** block proposal, vote participation, profile-conformant verification.
- **Q-Sentry:** public edge P2P handling and validator allowlist forwarding.
- **Q-Witness:** independent verification and evidence stream production.
- **Q-Archivist:** immutable history retention, forensic query support.
- **Q-Observer:** telemetry and anomaly signal surface (non-consensus authority).

Role boundaries are policy-driven and must not permit validator key reuse on internet-facing nodes.

## Consensus Profile Strategy

AOXC-Q rollout tracks:

- `q_profile_v1` — classical hardened baseline.
- `q_profile_v2` — hybrid verification (classical + PQ).
- `q_profile_v3` — PQ-preferred policy path after governance activation.

Unknown, malformed, or unauthorized profile payloads are rejected before state transition.

## Protocol Requirements

1. Consensus-visible profile identifiers are mandatory in profile-bound structures.
2. Handshake and peer negotiation must reject downgrade attempts.
3. Admission pipeline enforces cheap checks before expensive verification.
4. Profile activation/deprecation is governance-bound and rollback-capable.

## Testnet Rollout Controls

Before promotion beyond testnet:

- deterministic simulation across mixed profile vectors must converge;
- downgrade and replay-path rejection telemetry must be retained;
- runbook-driven rollback rehearsal must complete with reproducible artifacts;
- operational evidence package must include profile compatibility and saturation reports.

## Mainnet Promotion Criteria

Mainnet activation for AOXC-Q requires:

- testnet gate and readiness gate stability over sustained soak windows;
- release-line and version governance consistency (`Cargo.toml`, `configs/version-policy.toml`, readiness surfaces);
- explicit residual-risk statement and operator sign-off.

## Versioning

- Consensus release line: `AOXC-Q-v0.2.0`
- Workspace version baseline: `0.2.0-aoxcq`

Any subsequent AOXC-Q release must maintain explicit migration notes and compatibility impact disclosure.
