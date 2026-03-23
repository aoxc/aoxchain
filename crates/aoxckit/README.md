# AOXCKIT

**Documentation Version:** `aoxc.v.0.1.0-testnet.1`
**Cargo-Compatible Version:** `0.1.0-testnet.1`

## Executive Summary
AOXCKIT is the cryptographic operations toolkit for operator workflows such as certificate generation, revocation, registry management, and actor identity tooling.

## Architectural Overview
This component is part of the AOX Chain production roadmap and is documented as a reviewable subsystem rather than a placeholder package. The goal of this README is to give enough context that an engineer, auditor, or operator can understand why the crate exists, what code families it owns, and where the main security boundaries live.

## Main Code Areas
- `keyforge`: command modules for keys, certs, revocation, actor IDs, registry, and supporting utilities.

## Security and Audit Focus
Human-facing cryptographic tooling must be deterministic, auditable, and difficult to misuse.

Reviewers should additionally confirm the following before promotion.
- Interfaces remain deterministic and version-aligned with the workspace baseline.
- Inputs are validated before affecting durable state or privileged behavior.
- Tests cover both expected behavior and hostile or malformed scenarios.
- Operational assumptions are mirrored in the corresponding `READ.md` and `VERSION.md` files.

## Integration Notes
This README is intentionally paired with a folder-specific `READ.md` and `VERSION.md`. The README explains the subsystem at a high level, the READ document explains the production audit expectations in more depth, and the VERSION document defines the mandatory release-discipline rules for future changes.

## Release Status
Current subsystem baseline: `aoxc.v.0.1.0-testnet.1` / `0.1.0-testnet.1`.
