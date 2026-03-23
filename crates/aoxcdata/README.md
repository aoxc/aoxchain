# AOXCDATA

**Documentation Version:** `aoxc.v.0.0.0-alpha.3`
**Cargo-Compatible Version:** `0.0.0-alpha.3`

## Executive Summary
AOXCDATA stores deterministic data-plane abstractions for state and contract persistence.

## Architectural Overview
This component is part of the AOX Chain production roadmap and is documented as a reviewable subsystem rather than a placeholder package. The goal of this README is to give enough context that an engineer, auditor, or operator can understand why the crate exists, what code families it owns, and where the main security boundaries live.

## Main Code Areas
- `store`: key-value, column-family, and state-tree abstractions.
- `contracts`: storage-facing contract state services.

## Security and Audit Focus
Persistence behavior must be reproducible, bounded, and resilient to schema drift or accidental data corruption.

Reviewers should additionally confirm the following before promotion.
- Interfaces remain deterministic and version-aligned with the workspace baseline.
- Inputs are validated before affecting durable state or privileged behavior.
- Tests cover both expected behavior and hostile or malformed scenarios.
- Operational assumptions are mirrored in the corresponding `READ.md` and `VERSION.md` files.

## Integration Notes
This README is intentionally paired with a folder-specific `READ.md` and `VERSION.md`. The README explains the subsystem at a high level, the READ document explains the production audit expectations in more depth, and the VERSION document defines the mandatory release-discipline rules for future changes.

## Release Status
Current subsystem baseline: `aoxc.v.0.0.0-alpha.3` / `0.0.0-alpha.3`.
