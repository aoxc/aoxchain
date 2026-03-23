# docs/

**System Version Baseline:** `aoxc.v.0.1.1-akdeniz`

## Mission
This folder is the canonical documentation surface for AOXC architecture, release planning, operator runbooks, readiness criteria, and audit evidence.

## What should be found here

- architecture decisions and technical analysis,
- production/readiness roadmaps,
- deterministic testnet and operational runbooks,
- versioning and release semantics,
- incident/on-call guidance,
- validation criteria and evidence expectations.

## Governance Rules
1. Keep documentation aligned with current system behavior and release naming.
2. Update release evidence whenever operator-facing or protocol-facing behavior changes.
3. Distinguish clearly between implemented behavior, target state, and unresolved blockers.
4. Prefer deterministic commands and reproducible examples.
5. Record residual risk explicitly when a subsystem is not fully production-closed.
6. Keep release naming consistent across docs, manifests, and top-level readmes.

## Versioning Notes

- Canonical documentation label: `aoxc.v.0.1.1-akdeniz`
- Cargo-compatible version baseline: `0.1.1-akdeniz`
- Human-facing release label: `AOXC v0.1.1-akdeniz`

## Suggested reading order

1. `AKDENIZ_RELEASE_BASELINE.md`
2. `SUMMARY.md`
3. `MAINNET_READINESS_CHECKLIST.md`
4. `REAL_NETWORK_VALIDATION_RUNBOOK_TR.md`
5. `AOXC_REAL_VERSIONING_AND_RELEASE_ROADMAP_TR.md`
