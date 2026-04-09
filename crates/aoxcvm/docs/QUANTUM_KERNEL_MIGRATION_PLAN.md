# QUANTUM KERNEL MIGRATION PLAN

This document defines the implementation roadmap to move AOXCVM from a hybrid-capable posture to a post-quantum-strict kernel and protocol surface.

## 1) Baseline Assessment (Current Repository State)

The repository already contains core post-quantum building blocks:

- algorithm registry with `ml-dsa-*` signatures and profile-aware authorization policies,
- deterministic hybrid bundle validation and migration policy helpers,
- post-quantum-strict policy mode support,
- quantum-hardened digest APIs for deterministic hashing surfaces.

This means the migration is not a greenfield redesign; it is a governance-controlled hardening and deprecation program.

## 2) Target State

AOXCVM target state is:

- `PostQuantumStrict` as the only accepted transaction signer profile on production networks,
- no consensus-visible fallback accepting classical-only signer sets,
- deterministic verification paths for post-quantum signatures across mempool, block validation, and replay,
- operational tooling, genesis formats, and release evidence aligned with PQ-only assumptions.

## 3) Program Constraints

1. Determinism must remain invariant across all cryptographic profile transitions.
2. Consensus-visible structures must remain versioned and fail-closed.
3. Any compatibility break must ship with explicit migration and rollback governance.
4. Performance regressions from larger signature artifacts must be measured and bounded.

## 4) Delivery Plan

### Phase A — Kernel and Auth Closure

Objective: complete internal kernel and auth hardening for PQ-only enforcement.

Work items:

- remove duplicate or legacy kernel code paths that can cause configuration ambiguity,
- enforce a single canonical `KernelSecurityLevel` preset path in `vm/kernel.rs`,
- require `PostQuantumStrict` for production auth profile defaults in runtime configuration,
- assert fail-closed verification behavior for classical signers in all admission paths.

Exit criteria:

- kernel compiles and tests pass with a single authoritative security-level configuration path,
- auth tests prove rejection of classical-only envelopes under strict mode,
- no runtime path silently downgrades to legacy signer acceptance.

### Phase B — Consensus and Transaction Surface Enforcement

Objective: make PQ policy consensus-visible and non-optional.

Work items:

- version transaction envelope and signer-set metadata to encode mandatory PQ policy,
- reject blocks containing classical-only signer sets at validation time,
- add telemetry counters for rejected downgrade or mixed-policy submissions,
- pin migration-state transitions to explicit governance events.

Exit criteria:

- mixed-policy network simulations converge deterministically,
- downgrade attempts are rejected and observable in operator telemetry,
- replay from genesis to tip produces identical acceptance/rejection outcomes.

### Phase C — State, Tooling, and Genesis Migration

Objective: move accounts and operator workflows fully to PQ credentials.

Work items:

- add deterministic account-key migration flows preserving authorization continuity,
- require PQ participation in key-rotation transactions across all supported profiles,
- update genesis manifests and validator identity tooling for PQ-only onboarding,
- provide deterministic backfill scripts for legacy account sets.

Exit criteria:

- validator and operator bootstrap procedures are PQ-only,
- key rotation and recovery workflows preserve continuity without classical fallback,
- migration evidence package includes before/after account-set validation.

### Phase D — Performance and Capacity Re-Baselining

Objective: absorb PQ artifact size/verification cost into production limits.

Work items:

- benchmark signature verification throughput and memory footprints under PQ loads,
- tune gas, payload size, and mempool admission bounds for stable latency,
- validate block propagation and consensus timing under realistic PQ transaction mixes,
- refresh capacity planning documentation with measured limits.

Exit criteria:

- stress and soak tests pass against updated resource envelopes,
- no unbounded memory or latency growth in validation-critical paths,
- release gate includes signed benchmark evidence artifacts.

### Phase E — Mainnet Activation and Legacy Deprecation

Objective: activate PQ-only policy with explicit deprecation governance.

Work items:

- schedule and ratify the deprecation height/epoch for classical signatures,
- execute staged testnet rehearsals and rollback drills,
- lock production policy to PQ-only post-activation,
- publish post-activation residual risk and assumption review.

Exit criteria:

- activation/rehearsal artifacts are reproducible,
- rollback decision tree is validated but not required in final activation,
- classical signer acceptance is cryptographically and procedurally retired.

## 5) Verification Matrix (Mandatory)

- unit/property tests for auth scheme and bundle validation,
- deterministic replay tests across profile transition boundaries,
- integration tests for consensus admission and block rejection behavior,
- adversarial tests for downgrade injection, signer-set malleability, and envelope mutation,
- operator-drill evidence for key rotation, incident rollback, and chain continuity.

## 6) Immediate Next Actions

1. Stabilize kernel configuration surfaces and remove redundant definitions.
2. Promote PQ-only profile checks to default network admission policy.
3. Add transition tests that model full lifecycle: legacy -> hybrid -> strict PQ.
4. Publish governance schedule for disabling legacy signer acceptance.
