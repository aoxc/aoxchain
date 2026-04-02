# AOXCVM Phase-2 Runtime Expansion Layer

## Objective

Phase-2 extends the runtime surface **without weakening** the Phase-1 kernel guarantees:

- kernel lifecycle remains unchanged,
- `execute(...)` remains the primary execution entry,
- host boundary remains strict,
- admission stays fail-closed.

This phase adds contract-law metadata and resolver-level execution classification.

## Delivered Baseline in This Step

### 1) Contract Law Data Model (initial)

`aoxcontract` now includes:

- `ContractClass` (`Application`, `System`, `Governed`, `Package`, `PolicyBound`),
- `CapabilityProfile`,
- `PolicyProfile`,
- `ExecutionProfile`.

### 2) Manifest-level Runtime Binding

Each `ContractManifest` now carries an `execution_profile`.

- The profile is initialized via `ExecutionProfile::phase2_default(&vm_target)`.
- Validation ensures `execution_profile.vm_target == manifest.vm_target`.

### 3) Resolver-level Expansion

`aoxcvm::contracts::resolver::resolve_runtime_binding(...)` now emits a Phase-2
class-aware execution profile reference:

- `phase2-application`,
- `phase2-system`,
- `phase2-governed`,
- `phase2-package`,
- `phase2-policy-bound`.

The runtime binding payload now includes `resolved_profile` so consumers can
inspect class/capability/policy context directly.

### 4) Capability + Policy Enforcement in Resolver

Resolver admission now fail-closes when runtime law and manifest policy diverge.

Examples:

- profile `storage_read=true` requires policy `StorageRead`,
- profile `storage_write=true` requires policy `StorageWrite`,
- profile `governance_hooks=true` requires policy `GovernanceBound`,
- profile `restricted_syscalls=true` requires policy `PrivilegedHook`,
- profile policy `review_required=true` requires manifest policy review,
- profile policy `governance_activation_required=true` requires governance activation mode.

### 5) Contract-Class Behavior Matrix (initial)

Class-sensitive guardrails are now active in resolver policy enforcement:

- `Application`: cannot request governed/restricted authority capabilities,
- `System`: must remain `review_required=true`,
- `Governed`: must require governance activation,
- `Package`: max one entrypoint,
- `PolicyBound`: must declare `restricted_to_auth_profile`.

### 6) SDK Builder Alignment

The contract builder now supports explicit Phase-2 profile shaping:

- `with_execution_profile(...)`,
- `with_contract_class(...)`,
- `with_capability_profile(...)`,
- `with_policy_profile(...)`.

## Non-Goals (still unchanged)

This step does **not**:

- bypass kernel enforcement,
- introduce state-write side channels outside existing syscall paths,
- open unrestricted native extension paths,
- relax deterministic or metering boundaries.
