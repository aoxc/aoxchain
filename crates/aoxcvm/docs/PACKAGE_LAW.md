# AOXCVM Package Law

## Purpose

Package law defines constitutional rules for package publication, compatibility, trust,
and lifecycle progression.

## Package constitutional classes

Phase-3 package law supports at least:
- Application,
- System,
- Governed,
- PolicyBound,
- Constitutional,
- AuthorityScoped,
- SettlementAware.

Class selection determines allowed capabilities, governance boundaries,
and upgrade/promotion requirements.

## Publication law

Package publication requires:
- canonical package identity,
- deterministic manifest validation,
- class-compatible capability profile,
- profile-bound publisher authority,
- trust-domain declaration.

Non-conforming publication attempts are rejected and receipted.

## Dependency graph law

- Dependency edges must be explicit and acyclic where required by class law.
- Version compatibility constraints must be machine-checkable.
- Restricted dependencies for constitutional/system packages must be deny-by-default.

## Versioning and immutability

- Published package versions are immutable.
- Mutations require version increments with governed compatibility checks.
- Immutable law surfaces must never be rewritten in place.

## Promotion lifecycle

A package promotion lifecycle is canonicalized:
- admitted,
- staged,
- governed,
- active,
- deprecated,
- retired.

Promotion transitions are lane-governed and produce auditable receipts.

## Trust domains

Package trust domains define:
- signer authority scope,
- dependency allowlist policy,
- promotion authority,
- settlement/interop exposure constraints.

## Enforcement requirements

Package law must be enforced at:
- manifest admission,
- dependency resolution,
- runtime loading,
- upgrade coordination,
- governance action application.
