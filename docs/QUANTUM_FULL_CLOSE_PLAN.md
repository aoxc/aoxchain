# AOXChain Quantum-Full Near-Closure Plan

This document defines the concrete, reviewable path to move AOXChain from "quantum-ready by governance" to "quantum-full near-closure" for release decision support.

## 1. Objective

Reach a state where direct post-quantum operation is operationally credible under deterministic, fail-closed, and evidence-backed controls.

This is a near-closure target, not an absolute-security claim.

## 2. Entry Conditions

The plan starts only when all conditions below hold:

1. kernel profile authority is already consensus-visible;
2. cryptographic profile identifiers are versioned and explicitly validated;
3. testnet baseline gates are green for the active release line.

## 3. Near-Closure Definition

"Quantum-full near-closure" is satisfied only when all tracks below pass.

### 3.1 Kernel Policy Closure

- unknown or unsupported profile payloads are rejected before settlement;
- profile activation/deprecation windows are policy-defined and version-bounded;
- classical-only acceptance paths are removed or constrained by explicit dual-profile windows.

### 3.2 Network and Handshake Closure

- peer negotiation fails closed on profile mismatch;
- downgrade attempts are rejected and surfaced in operator telemetry;
- handshake behavior is deterministic across supported node roles.

### 3.3 Migration Closure

- validator/operator key transition path is deterministic and documented;
- persisted consensus artifacts have deterministic migration or controlled reset rules;
- migration rehearsal artifacts are retained per release candidate.

### 3.4 Rollback Closure

- rollback path is explicit, version-bounded, and rehearsed;
- rollback-by-config-drift is prohibited by policy and validation checks;
- incident drill evidence includes rollback timing and safety outcomes.

### 3.5 Evidence and Gate Closure

- required gate commands are reproducible and retained with artifact references;
- mixed-profile rejection proofs are produced for candidate cutovers;
- final readiness package links commit, command logs, artifacts, and risk statement.

## 4. Required Evidence Package

A candidate claiming near-closure must include:

1. deterministic simulation matrix for mixed profile peers;
2. downgrade rejection proof bundle;
3. migration drill logs and artifact hashes;
4. rollback rehearsal records;
5. operator runbook validation outputs;
6. updated residual risk statement.

## 5. Acceptance Rule

No production promotion decision may claim quantum-full near-closure unless every track in Section 3 and every evidence item in Section 4 is present, reproducible, and linked to the candidate commit.

## 6. Non-Goals

This plan does not claim:

- unconditional cryptographic permanence;
- legal/regulatory guarantees;
- infallibility against unknown future cryptanalytic advances.

## 7. Maintenance Rule

Any change affecting profile governance, migration semantics, handshake behavior, rollback guarantees, or evidence generation must update this document in the same change set.
