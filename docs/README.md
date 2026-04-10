# AOXChain Docs Surface

This directory contains the authoritative documentation system for AOXChain engineering, operations, and audit readiness.

## Documentation Model

- `src/` — mdBook chapters used as the canonical reading path.
- `testing/` — testing matrix, coverage posture, and critical invariants.
- `AOXCHUB_FULL_SPEC.md` — AOXCHub product-level specification.
- `ADVANCED_NODE_ROLE_BLUEPRINT.md` — full role/layer/plane topology with policy-gated activation.
- `NAMING_VERSIONING_SIMPLIFICATION_PLAN.md` — naming/versioning governance and migration baseline.
- `GENESIS_IDENTITY_CHECKLIST.md` — deterministic chain identity checklist from registry to genesis/runtime.
- `OS_COMPATIBILITY.md` — cross-platform host/container compatibility contract (Linux, NixOS, macOS, Windows, Docker).
- `bootstrap/BOOTSTRAP_RUNBOOK.md` — deterministic bootstrap phases, gate model, failure taxonomy, and closure evidence contract.
- `FULL_NODE_GUIDE.md` — step-by-step full-node setup, join flow, and re-bootstrap safety guidance.
- `API_REFERENCE.md` — implementation-aligned HTTP/gRPC API surface and usage examples.
- `API_KERNEL_SECURITY_BLUEPRINT.md` — API-first security kernel completion contract and cross-repository handoff checklist for downstream client implementations.
- `CLIENT_REPOSITORY_API_INTEGRATION_SPEC.md` — exact implementation contract for the separate client repository consuming AOXChain APIs.
- `PQ_COMPATIBILITY_GAP_AND_IMPLEMENTATION_PLAN.md` — implementation-aligned Dilithium/Falcon compatibility gap analysis and staged delivery plan.
- `QUANTUM_ACCOUNT_MANAGEMENT_BLUEPRINT.md` — post-quantum authority/account blueprint covering scheme agility, policy roots, replay/recovery controls, validator integration, and phased validation-kernel rollout.

## Documentation Rules

- Keep documents implementation-aware and reviewable.
- Prefer explicit constraints and operational guidance over narrative prose.
- Update docs in the same change stream when behavior, architecture, or policy changes.
- Preserve consistent terminology for brand, ticker, release line, workspace version, chain identity, and crypto profile.

## Build Commands

```bash
mdbook build docs
mdbook serve docs -n 0.0.0.0 -p 3000
```
