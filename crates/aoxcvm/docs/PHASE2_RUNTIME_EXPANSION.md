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

## Non-Goals (still unchanged)

This step does **not**:

- bypass kernel enforcement,
- introduce state-write side channels outside existing syscall paths,
- open unrestricted native extension paths,
- relax deterministic or metering boundaries.
