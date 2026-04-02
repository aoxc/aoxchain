# AOXCVM Phase-2 Runtime Expansion Layer

> Canonical constitutional law tables for this phase are defined in
> `PHASE2_EXECUTION_LAW.md`.
>
> Final acceptance checklist and completion ruling are tracked in
> `PHASE2_FINAL_STATUS.md`.

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

### 3) Resolver-level Expansion + Enforcement

`aoxcvm::contracts::resolver::resolve_runtime_binding(...)` now emits a Phase-2
class-aware execution profile reference:

- `phase2-application`,
- `phase2-system`,
- `phase2-governed`,
- `phase2-package`,
- `phase2-policy-bound`.

The runtime binding payload now includes `resolved_profile` so consumers can
inspect class/capability/policy context directly.

Resolver now applies fail-closed checks for:

- `PolicyProfile.review_required` (cannot downgrade global required review),
- `PolicyProfile.governance_activation_required` (class-constrained),
- `PolicyProfile.restricted_to_auth_profile` (required for `PolicyBound` only),
- capability-class mismatches (forbidden feature elevation).

### 4) Contract-class behavior matrix

| Contract class | Allowed features | Forbidden features | Admission expectations |
|---|---|---|---|
| `Application` | storage read/write, package dependency access, optional restricted syscalls | registry access, governance hooks, metadata mutation, upgrade authority | user-call compatible, no governance-only hooks |
| `System` | all capabilities as configured | none at resolver level | system/governance lanes accepted |
| `Governed` | governance hooks, registry access, metadata mutation, storage read/write | upgrade authority (direct escalation) | governance activation required when governance hooks are enabled |
| `Package` | storage read, package dependency access | storage write, registry access, governance hooks, metadata mutation, upgrade authority | package publishing and deterministic load only |
| `PolicyBound` | storage read/write, package dependency access, restricted syscalls | governance hooks, metadata mutation, upgrade authority | must declare non-empty restricted auth profile and restricted syscalls |

### 5) Runtime admission binding

`vm::admission::validate_phase2_admission(...)` enforces that:

- governance-required profiles run on governance/system transaction kinds,
- restricted auth profiles must be present and match active auth profile id.

## Non-Goals (still unchanged)

This step does **not**:

- bypass kernel enforcement,
- introduce state-write side channels outside existing syscall paths,
- open unrestricted native extension paths,
- relax deterministic or metering boundaries.
