# AOXC Audit Companion — v0.1.1-akdeniz

This file is the audit-oriented companion to the main `README.md`.

## Release identity

- **Release label:** `AOXC v0.1.1-akdeniz`
- **Cargo version baseline:** `0.1.1-akdeniz`
- **Documentation baseline:** `aoxc.v.0.1.1-akdeniz`

## Audit intent

This repository should be reviewed as a deterministic, multi-crate blockchain workspace with explicit trust boundaries:

- cryptographic identity and custody,
- consensus correctness and finality,
- operator tooling and runtime persistence,
- network transport and observability,
- release discipline and evidence retention.

## What “Akdeniz” should mean

The `akdeniz` baseline should signal a named release train with:

1. coherent versioning across workspace metadata and docs,
2. traceable tests for operator/node bootstrap flows,
3. explicit runbooks for operational validation,
4. documented remaining blockers rather than hidden assumptions.

## Required release evidence

- commands executed,
- tests passed or skipped,
- documentation updated,
- known limitations recorded,
- commit SHA linked to release notes.

## Primary references

- `README.md`
- `docs/src/AKDENIZ_RELEASE_BASELINE.md`
- `docs/src/MAINNET_READINESS_CHECKLIST.md`
- `docs/src/REAL_NETWORK_VALIDATION_RUNBOOK_TR.md`
- `docs/src/AOXC_REAL_VERSIONING_AND_RELEASE_ROADMAP_TR.md`
