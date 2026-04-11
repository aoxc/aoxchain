# AOXChain Technical Contract

This document is the repository-wide technical contract for deterministic, policy-governed, and migration-safe operation.

## 1. Mission Contract

AOXChain must preserve all of the following properties:

- deterministic state transitions,
- fail-closed validation,
- policy-based authority semantics,
- cryptographic agility through explicit profile governance,
- evidence-backed readiness declarations.

## 2. Non-Negotiable Invariants

1. Identical canonical input must produce identical canonical output.
2. Validation must complete before execution may begin.
3. Consensus truth is kernel-owned and cannot be overridden by service or operations layers.
4. Consensus-relevant cryptography is profile-tagged and policy-bound.
5. `policy_root` and `recovery_root` remain logically independent.
6. Replay protection is domain-separated and migration-safe.
7. Rotation and migration are explicit state transitions, not ad-hoc operational events.
8. Wallet, validator, governance, recovery, and transport key domains are strictly non-overlapping.
9. Readiness claims are invalid without reproducible evidence.

## 3. Authority Contract

Validation pipeline (normative order):

`actor -> scheme_id -> policy_root -> proof_bundle -> replay check -> execute`

Minimum actor classes:

- account authority,
- validator authority,
- governance authority.

All actor classes must support controlled key rotation, scheme migration, and policy rotation via explicit authorization semantics.

## 4. Cryptographic Profile Contract

- profile identifiers are first-class protocol data,
- profile activation and deprecation are governance controlled,
- unsupported profiles are rejected fail-closed,
- hybrid windows are explicit, bounded, and evidence-gated,
- hidden fallback to deprecated classical profiles is prohibited.

## 5. Layer Responsibilities

### Kernel
Owns authority interpretation, consensus truth, profile enforcement, replay semantics, and settlement admission.

### Execution
Performs deterministic computation under kernel admission and deterministic cost policy.

### Services
Provides network, RPC, and storage transport; all ingress remains untrusted until validated.

### Operations
Provides orchestration and evidence production; cannot mutate canonical protocol truth outside approved transitions.

## 6. Mandatory Gate Baseline

At minimum, readiness evaluation includes:

```bash
make build
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

If any required gate fails (or is skipped without approved exception), readiness state is `NOT_READY`.

## 7. Architecture-Sensitive Change Rule

Any architecture-sensitive change must update, in the same change set:

1. `ARCHITECTURE.md` (boundary and responsibility impact),
2. `ROADMAP.md` (phase/checklist impact),
3. `TESTING.md` (validation/evidence impact),
4. `KEY_MANAGEMENT.md` (key lifecycle/domain impact, if applicable).

Implementation-only architectural drift is disallowed.

## 8. Audit Evidence Minimum

Audit packages for sensitive changes must include:

- executed commands,
- pass/fail outcomes,
- retained artifacts mapped to commit SHA,
- explicit residual-risk statement for deferred controls.

## 9. License and Liability Context

AOXChain is distributed under the MIT License on an **"AS IS"** basis, without warranty or liability assumptions by maintainers or contributors except where prohibited by law.
