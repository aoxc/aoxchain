# AOX Chain Crate Portfolio

**Documentation Version:** `aoxc.v.0.0.0-alpha.3`
**Cargo-Compatible Version:** `0.0.0-alpha.3`

## Executive Summary
This document explains how the Rust crate portfolio is partitioned, why each crate exists, and how reviewers should reason about security ownership across the workspace.

## Architectural Overview
This component is part of the AOX Chain production roadmap and is documented as a reviewable subsystem rather than a placeholder package. The goal of this README is to give enough context that an engineer, auditor, or operator can understand why the crate exists, what code families it owns, and where the main security boundaries live.

## Main Code Areas
- `aoxcore`: canonical protocol types, identity, genesis, state, receipts, contracts, and transaction models.
- `aoxcunity`: consensus rules, quorum logic, safety constraints, validator rotation, finality, and recovery.
- `aoxcvm`: multi-lane execution orchestration for native, EVM, Wasm, Cardano, Sui Move, and system flows.
- `aoxcnet` / `aoxcrpc`: networking, gossip, discovery, HTTP/gRPC/WebSocket APIs, and request middleware.
- `aoxcmd`: operator binary, CLI flows, runtime assembly, telemetry, and node lifecycle orchestration.
- Supporting crates (`aoxcdata`, `aoxconfig`, `aoxcontract`, `aoxcsdk`, `aoxcmob`, `aoxckit`, `aoxclibs`, `aoxcexec`, `aoxchal`, `aoxcenergy`, `aoxcai`) cover storage, configuration, contract policy, SDKs, mobile security, crypto tooling, shared utilities, execution interfaces, hardware-aware helpers, economics, and AI governance.

## Security and Audit Focus
Reviews should confirm that crate boundaries reflect real trust boundaries. If a security assumption crosses crate boundaries, both sides must document the contract and provide test evidence.

Reviewers should additionally confirm the following before promotion.
- Interfaces remain deterministic and version-aligned with the workspace baseline.
- Inputs are validated before affecting durable state or privileged behavior.
- Tests cover both expected behavior and hostile or malformed scenarios.
- Operational assumptions are mirrored in the corresponding `READ.md` and `VERSION.md` files.

## Integration Notes
This README is intentionally paired with a folder-specific `READ.md` and `VERSION.md`. The README explains the subsystem at a high level, the READ document explains the production audit expectations in more depth, and the VERSION document defines the mandatory release-discipline rules for future changes.

## Release Status
Current subsystem baseline: `aoxc.v.0.0.0-alpha.3` / `0.0.0-alpha.3`.
