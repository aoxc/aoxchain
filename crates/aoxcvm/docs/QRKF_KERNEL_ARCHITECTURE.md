# AOXCVM QRKF Kernel Architecture

## Scope

This document defines AOXC Quantum-Resilient Key Fabric (QRKF) as a kernel-enforced identity and authority system, not a wallet-only key generator.

## Design Goal

Eliminate single-point authorization assumptions:

- no single-key trust,
- no single-algorithm trust,
- no single-lane trust,
- no recovery bypass path.

## Kernel-Mandatory Surfaces

The following are protocol/kernel surfaces and must not be delegated to optional adapters:

1. **Identity envelope semantics**: `realm`, `profile_id`, `epoch`, canonical serialization.
2. **Crypto profile registry**: profile activation/deprecation lifecycle and bounded verifier requirements.
3. **Authority lane policy**: deterministic lane thresholds and mandatory lane classes.
4. **Epoch continuity rules**: predecessor linkage, anti-downgrade, anti-replay epoch progression.
5. **Recovery constitution hooks**: veto/timelock/freeze semantics and forced post-recovery rotation.
6. **Evidence requirements**: canonical fingerprints, rotation artifacts, and continuity manifests.

## Adapter/Extension Surfaces

The following are intentionally adapter-driven and can evolve without protocol format rewrites:

- algorithm families,
- witness schema versions,
- verifier adapters per profile family,
- hardware custody modes,
- institutional/compliance lane variants.

## Canonical Baseline Objects

QRKF kernel primitives are represented by:

- `KeyRealm`,
- `CryptoProfileId`,
- `AuthorizationLane`,
- `LanePolicy`,
- `EpochKeyBundle`.

These objects provide deterministic wire IDs, lane-threshold checks, and continuity-link validation.

## Production Closure Mapping

QRKF closure is complete only when:

- kernel-mandatory surfaces are ratified in protocol governance,
- adapter surfaces are versioned in profile registry policy,
- continuity and recovery artifacts are emitted for every key epoch transition,
- independent audit validates both kernel constraints and adapter boundaries.
