# AOXChain Scope Statement

## Purpose

This statement defines the intended engineering, operational, and documentation scope of the AOXChain repository. Its purpose is to establish clear boundary conditions for development, review, audit preparation, and release-readiness claims.

## In Scope

The following areas are within the active scope of this repository:

- deterministic Layer 1 engineering, including consensus, execution, networking, and state-transition primitives;
- operator tooling, automation, and control workflows required to run, validate, inspect, and audit environments;
- environment profiles, deterministic fixtures, and reproducible test or deployment inputs used to support controlled validation;
- release evidence, readiness artifacts, and production-closure materials used to substantiate operational readiness claims;
- repository documentation required for engineering governance, security coordination, validation traceability, and institutional review;
- trusted interface surfaces, configuration boundaries, and execution assumptions directly maintained within the AOXChain workspace.

## Out of Scope

The following matters are expressly outside the guaranteed scope of this repository:

- any representation, warranty, or guarantee of production fitness, uninterrupted operation, merchantability, or fitness for a particular purpose;
- legal, regulatory, tax, accounting, or jurisdiction-specific compliance assurances;
- financial suitability, investment outcomes, custody guarantees, or protection from loss;
- custodial services, regulated financial operations, or contractual service-level obligations unless separately and explicitly established outside this repository;
- unconditional backward compatibility across experimental, provisional, or actively evolving interfaces during active development.

## Sensitive Change Classes

The following change classes require heightened review, explicit rationale, and corresponding documentation updates:

- consensus safety, liveness, quorum, finality, or fork-choice behavior;
- deterministic execution, state-transition logic, or reproducibility semantics;
- key lifecycle management, signer workflows, authorization rules, or trust boundaries;
- persisted data formats, serialization rules, database assumptions, or migration logic;
- public RPC, CLI, API, or integration contract changes;
- release controls, validation workflows, or evidence-generation mechanisms that affect auditability or operational assurance.

For these change classes, implementation review alone is insufficient. The change record must also reflect the intended behavioral impact, validation strategy, and any compatibility consequences.

## Compatibility Policy

Compatibility is governed at the release-line level through explicit engineering review and evidence-based readiness controls. Breaking changes are permitted where justified by safety, determinism, maintainability, or architectural integrity objectives, but such changes must be explicitly declared in release documentation and supported by appropriate migration or operator guidance where applicable.

No implicit compatibility promise should be inferred for experimental interfaces, internal-only modules, or development-stage control surfaces unless such compatibility is expressly documented.

## Governance and Evidence Expectations

Scope claims, readiness claims, and operational assertions must be supported by reviewable evidence. Documentation should remain aligned with actual repository behavior, control flows, and release practices.

Claims that cannot be substantiated by retained evidence, reproducible procedures, or current repository state should be treated as non-authoritative.

## License and Liability Context

AOXChain is provided under the MIT License. All repository materials are provided on an **"as is"** basis, without warranties, guarantees, or liability assumptions by maintainers or contributors, except where such limitations are prohibited by applicable law.


## Quantum-Resilience Scope Addendum

The following items are explicitly in scope for the current roadmap cycle:

- cryptographic profile versioning across consensus-visible structures;
- hybrid migration controls (classical + PQ) with explicit deprecation windows;
- deterministic rollback and operator evidence requirements for profile transitions;
- VM (`aoxcvm`) cryptographic syscall governance and deterministic metering implications.

Out of scope for this cycle unless separately approved:

- unverifiable claims of absolute security (for example, "unbreakable");
- hidden fallback paths that bypass profile policy and downgrade protections.

Reference documents:
- `QUANTUM_ROADMAP.md`
- `QUANTUM_CHECKLIST.md`
