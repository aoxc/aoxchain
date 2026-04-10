# Versioning Policy

## Purpose

This document defines how AOXChain version state is advanced in a controlled, auditable, and Git-compatible workflow.

Current active release line target: `AOXC-QTR-V1` (communication label only).

Current workspace release version: `0.2.0-alpha.2`.

## Canonical Version Surfaces

AOXChain version state is governed by three canonical surfaces:

- Repository release version: `Cargo.toml` (`[workspace.package].version`)
- Machine-readable governance policy: `configs/version-policy.toml` (`[workspace].current`)
- Git release tag namespace: `v<workspace-version>`

All three surfaces must remain synchronized for release-grade operation.

## Versioning Model

AOXChain uses a hybrid model:

1. **Global workspace version** (single release identity)
   - Represents the release identity for the full repository.
   - Must advance when merged changes affect shipped behavior, operator behavior, or release artifacts.
2. **Component schema tracks**
   - Protocol/schema/policy tracks advance independently when a schema contract changes.
   - Schema bumps require compatibility rationale and migration posture.

Versioning is intentionally **not per-file** and **not per-folder by default**.

## Semantic Rules

### Global workspace SemVer rules

- `MAJOR`: compatibility-breaking protocol/API/storage/governance changes.
- `MINOR`: backward-compatible capability additions.
- `PATCH`: backward-compatible fixes, hardening, or operational corrections.
- Pre-release suffixes (for example `-alpha.1`) are allowed for non-final release lines.

### Controlled-channel contract

`configs/version-policy.toml` must preserve:

- `strategy = "global-workspace-version-with-component-schema-tracks"`
- `release_channel = "controlled"`

These keys are treated as release-governance controls and are verified by tests and gates.

## Git-Compatible Change Enforcement

Version governance is enforced through:

- Rust tests in `tests/src/version_governance.rs`.
- Shell gate: `scripts/validation/versioning_gate.sh`.
- Make target: `make versioning-gate` (also run by `make quality-release`).

The versioning gate verifies:

1. `Cargo.toml` workspace version equals `configs/version-policy.toml` current version.
2. Workspace version matches SemVer core format (`MAJOR.MINOR.PATCH[-PRERELEASE]`).
3. If `HEAD` is tagged, the tag must be exactly `v<workspace-version>`.
4. If version-sensitive files changed relative to the configured base ref, at least one canonical version surface must also change.
5. Workspace version must be greater than the latest `v*` Git tag unless explicitly overridden.

## Forced Version Advancement Policy

When a change touches version-sensitive engineering surfaces (for example crates, contracts, runtime configs, release scripts, core build controls, or governance tests), version advancement is mandatory unless a reviewer-approved exception is explicitly documented.

Default policy outcome:

- version-sensitive changes without version-surface updates are rejected by `versioning_gate.sh`.

Override path (exception only):

- set `AOXC_ALLOW_NON_INCREMENTAL_VERSION=1` when a non-incremental version is intentionally required for controlled maintenance flow.

## Standard Workflow

1. Apply implementation changes.
2. Decide required SemVer bump (`MAJOR`, `MINOR`, `PATCH`, optional pre-release suffix).
3. Update:
   - `Cargo.toml` `[workspace.package].version`
   - `configs/version-policy.toml` `[workspace].current`
4. If schema contracts changed, bump relevant schema track values and document compatibility posture.
5. Run:
   - `make versioning-gate`
   - `cargo test -p tests version_governance -- --nocapture` (or repository test matrix)
6. Create/validate release tags as `v<workspace-version>` in release workflow.

## Reviewer Checklist

For version-sensitive pull requests, reviewers should verify:

- Canonical version surfaces are synchronized.
- SemVer class is consistent with compatibility impact.
- Schema track bumps exist where contract formats changed.
- Versioning gate and governance tests are green.
- Release tag plan aligns with `v<workspace-version>`.

## Operator Guidance

If bump scope is ambiguous, prefer conservative upward bump and document rationale in pull request notes.
