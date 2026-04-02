# AOXCVM Upgrade Constitution

## Purpose

Upgrade law defines who can change what, through which governance lane, under which
compatibility and migration constraints.

## Upgrade authority model

Upgrade authority is class-scoped:
- constitutional surfaces: constitutional lane only,
- operational surfaces: operations lane within constitutional limits,
- emergency controls: emergency lane with expiry and review,
- package/application surfaces: delegated authority with profile binding.

## Mutable vs immutable surfaces

### Immutable by default

- historical receipts and proofs,
- canonical package version artifacts,
- finalized state commitments,
- constitutional immutability declarations.

### Mutable through law

- governed parameters,
- feature gates,
- syscall/policy registries,
- upgrade schedules,
- migration orchestration metadata.

## Upgrade lanes and approvals

Every upgrade operation must declare:
- lane,
- action class,
- target surface,
- required signer classes,
- quorum and veto policy,
- compatibility impact class.

## Compatibility law

Upgrade admission requires compatibility checks over:
- execution semantics,
- object and receipt schemas,
- package dependency compatibility,
- governance/state transition expectations.

Breaking changes require constitutional-level transition handling.

## Migration law

Migrations must be:
- deterministic,
- replay-safe,
- resumable with checkpointing where needed,
- auditable with start/stop/outcome provenance.

No migration may bypass state integrity verification.

## Safety rules

- fail closed on incomplete approvals,
- fail closed on unresolved compatibility signals,
- block partial commits for constitutional upgrade actions,
- require explicit rollback strategy for governed upgrades.
