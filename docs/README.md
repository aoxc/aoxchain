# AOXChain Docs Surface

This directory contains the authoritative documentation system for AOXChain engineering, operations, and audit readiness.

## Documentation Model

- `src/` — mdBook chapters used as the canonical reading path.
- `testing/` — testing matrix, coverage posture, and critical invariants.
- `AOXCHUB_FULL_SPEC.md` — AOXCHub product-level specification.
- `ADVANCED_NODE_ROLE_BLUEPRINT.md` — full role/layer/plane topology with policy-gated activation.
- `OS_COMPATIBILITY.md` — cross-platform host/container compatibility contract (Linux, NixOS, macOS, Windows, Docker).

## Documentation Rules

- Keep documents implementation-aware and reviewable.
- Prefer explicit constraints and operational guidance over narrative prose.
- Update docs in the same change stream when behavior, architecture, or policy changes.

## Build Commands

```bash
mdbook build docs
mdbook serve docs -n 0.0.0.0 -p 3000
```
