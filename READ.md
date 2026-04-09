# AOXChain Technical Contract (Reset)

This document is the repository-wide technical contract for deterministic, policy-governed, post-quantum-oriented operation.

## 1) Mission Contract

AOXChain must deliver:

- deterministic state transitions,
- fail-closed validation,
- policy-based authority semantics,
- cryptographic agility with explicit profile governance,
- evidence-backed readiness declarations.

## 2) Core Invariants

1. Identical canonical inputs must produce identical canonical outputs.
2. Validation must complete before execution is allowed.
3. Consensus truth is kernel-owned and cannot be overridden by service or operations layers.
4. All consensus-relevant cryptography must be profile-tagged and policy-bound.
5. `policy_root` and `recovery_root` must remain logically independent.
6. Replay protection must be domain-separated.
7. Migration and rotation are native state transitions, not ad-hoc operator events.
8. No production/readiness claim is valid without reproducible evidence.

## 3) Authority Model Contract

Validation pipeline:

`actor -> scheme_id -> policy_root -> proof_bundle -> replay check -> execute`

Minimum actor classes:

- account authority,
- validator authority,
- governance authority.

All actor classes must support controlled key rotation, scheme migration, and policy rotation under explicit authorization rules.

## 4) Cryptographic Profile Contract

- profile IDs are first-class protocol data,
- profile activation and deprecation are governance-controlled,
- unsupported profiles fail closed,
- hybrid windows are explicit, time-bounded, and evidence-gated,
- hidden classical fallback paths are prohibited.

## 5) Layer Responsibilities

### Kernel
Owns authority model, consensus truth, profile policy enforcement, replay semantics, and settlement admission.

### Execution
Runs deterministic computation under kernel acceptance decisions and deterministic cost rules.

### Services
Provides network/RPC/storage/config transport and must treat all ingress as untrusted until validated.

### Operations
Provides orchestration, diagnostics, and evidence production; cannot mutate protocol truth outside approved policy transitions.

## 6) Required Gate Baseline

At minimum, readiness evaluation must include:

```bash
make build
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

If any required gate fails or is skipped without approved exception, readiness status is `NOT_READY`.

## 7) Architecture-Change Rule

Any architecture-sensitive change must update, in the same change set:

1. `ARCHITECTURE.md` (responsibility and trust-boundary impact),
2. `ROADMAP.md` (phase/checklist impact),
3. `TESTING.md` (validation scope and required evidence).

Implementation-only architecture drift is not allowed.

## 8) License and Liability Context

AOXChain is provided under the MIT License on an "as is" basis, without warranty or liability assumptions by maintainers or contributors except where restricted by applicable law.
