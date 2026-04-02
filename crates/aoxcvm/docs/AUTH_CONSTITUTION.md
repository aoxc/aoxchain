# AOXCVM Auth Constitution

## Purpose

This document defines canonical authentication law for the Phase-3 constitutional runtime.
Authentication is not a utility surface. It is a constitutional authority boundary.

## Constitutional guarantees

1. Every privileged runtime action must be attributable to a typed auth profile.
2. Every profile must resolve through canonical registry state and versioned policy.
3. Authorization decisions must be deterministic, replayable, and auditable.
4. Unsupported, ambiguous, or incomplete auth states must fail closed.

## Canonical model

### Auth profile identity

- `AuthProfileId` is typed and canonical.
- Profile records are versioned and immutable per version.
- Profile evolution is append-only through governed transition.

### Signer classes

Minimum signer classes:
- **SystemSigner**
- **GovernanceSigner**
- **OpsSigner**
- **ApplicationSigner**

Each class has class-specific authority surfaces and denial defaults.

### Threshold and quorum

- Threshold policy is part of profile law.
- Quorum and veto semantics are explicit per protected lane.
- Multi-signer constitutional paths are mandatory for constitutional-grade actions.

## Profile-bound execution law

At minimum, profile binding applies to:
- governance actions,
- runtime syscall families,
- package publication and promotion,
- registry mutation,
- upgrade initiation and approval,
- operational emergency controls.

## Registry law

The auth profile registry must provide:
- canonical lookup,
- version history,
- governance-controlled mutation,
- deterministic conflict resolution,
- provenance records for creation, rotation, and revocation.

## Evidence requirements

Auth decisions must emit provenance sufficient for:
- signer-class verification,
- threshold satisfaction proof,
- profile-version binding,
- replay-safe audit revalidation.

## Rejection law

Auth denials must emit structured reject reasons, including:
- unknown profile,
- wrong signer class,
- threshold/quorum not met,
- profile version mismatch,
- lane authorization violation.
