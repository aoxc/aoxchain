# AOXC v0.1.1-akdeniz Release Baseline

## Purpose

This document defines what the repository means when it says:

- **AOXC v0.1.1-akdeniz**
- **aoxc.v.0.1.1-akdeniz**
- **Cargo version `0.1.1-akdeniz`**

## Naming policy

The recommended convention is:

- human-facing release branding can use the `AOXC v0.1.1-akdeniz` form,
- Cargo manifests use `0.1.1-akdeniz`,
- docs and release notes can use `aoxc.v.0.1.1-akdeniz` when a canonical label is needed.

## Why not stay on alpha forever?

The repository already contains a release-roadmap argument that AOXC should not remain stuck in indefinite “alpha” wording.

Using `akdeniz`:

- keeps the version honest,
- preserves room for future iterations,
- gives operators a concrete release train name,
- avoids vague maturity language.

## Baseline requirements

For `v0.1.1-akdeniz`, the repository should aim to maintain:

1. coherent version references in workspace metadata and top-level docs,
2. documented operator bootstrap flow,
3. deterministic unit and targeted integration coverage for critical command-plane flows,
4. explicit readiness and runbook references,
5. known gaps documented rather than omitted.

## Release checklist

- update workspace/package versions,
- update top-level `README.md` and `READ.md`,
- update documentation baseline references,
- run focused verification commands,
- record commit SHA and outcomes.

## Non-goals

This baseline does **not** automatically mean:

- full mainnet readiness is complete,
- every crate has exhaustive integration coverage,
- state sync / recovery blockers are fully closed,
- every operational proof already exists.

Instead, it means the release line is named, consistent, documented, and tied to explicit evidence.
