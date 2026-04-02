# INVARIANTS

This document defines Phase-1 kernel invariants for `src/vm/phase1.rs`.
These invariants are compatibility-sensitive until an explicit version bump.

## Admission ordering (fail-closed)

`execute(...)` enforces deterministic admission order:

1. malformed payload check,
2. deterministic context validation,
3. auth verification,
4. object size and object verification,
5. host lifecycle (`checkpoint -> execute -> rollback/commit`).

The first failing gate terminates execution. Later gates must not run.

## Host boundary invariants

- No durable state mutation happens before `checkpoint`.
- Runtime failure must call `rollback` exactly once and must not call `commit`.
- Runtime success must call `commit` exactly once and must not call `rollback`.
- Host failures preserve provenance using typed errors:
  `HostCheckpoint`, `HostRollback`, and `HostCommit`.

## Outcome invariants

- `gas_used` mirrors receipt gas accounting.
- `journal_committed` is `true` only on successful commit path.
- `halt_reason` must match `vm_error`:
  - `Success` implies `vm_error == None`,
  - `VmError(_)` implies `vm_error == Some(_)`.
- `spec_version` reports the active `VmSpec` contract version.

## Spec derivation invariants

- `VmSpec::from_config(...)` is fail-closed on disabled VM targets.
- `max_object_bytes` is derived from contract artifact policy.
- `strict_mode` is derived from `review_required`.

## Phase-2 execution-law invariants

- Resolver and admission are both **fail-closed** for Phase-2 law; bypassing one layer must still be rejected by the next layer.
- `resolved_profile.vm_target` and manifest `vm_target` must remain equal.
- Forbidden class/capability combinations never reach execution.
- `PolicyBound` contracts must carry a canonical `restricted_to_auth_profile` value.
- Auth profile mismatch (`restricted_to_auth_profile` vs active runtime profile) never enters execution.
- Governance-required profiles are restricted to governance/system lanes at admission.
