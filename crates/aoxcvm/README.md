# AOXCVM

**Documentation Version:** `aoxc.v.0.0.0-alpha.3`
**Cargo-Compatible Version:** `0.0.0-alpha.3`

## Executive Summary
AOXCVM is the execution orchestration crate for AOX Chain. It coordinates host interfaces, routing, contract binding, and multiple virtual-machine lanes.

## Architectural Overview
This component is part of the AOX Chain production roadmap and is documented as a reviewable subsystem rather than a placeholder package. The goal of this README is to give enough context that an engineer, auditor, or operator can understand why the crate exists, what code families it owns, and where the main security boundaries live.

## Main Code Areas
- `lanes`: execution backends for native or external lane families such as EVM, Wasm, Cardano, and Sui Move.
- `host`: typed interfaces for state, block context, transaction context, storage, and receipts.
- `routing` and `contracts`: deterministic lane dispatch, contract installation, binding, and validation.
- `system` and `compatibility`: system contracts and external-runtime compatibility adapters.

## Security and Audit Focus
Execution isolation, host determinism, and clear routing invariants are the critical audit themes.

Reviewers should additionally confirm the following before promotion.
- Interfaces remain deterministic and version-aligned with the workspace baseline.
- Inputs are validated before affecting durable state or privileged behavior.
- Tests cover both expected behavior and hostile or malformed scenarios.
- Operational assumptions are mirrored in the corresponding `READ.md` and `VERSION.md` files.

## Integration Notes
This README is intentionally paired with a folder-specific `READ.md` and `VERSION.md`. The README explains the subsystem at a high level, the READ document explains the production audit expectations in more depth, and the VERSION document defines the mandatory release-discipline rules for future changes.

## Release Status
Current subsystem baseline: `aoxc.v.0.0.0-alpha.3` / `0.0.0-alpha.3`.
