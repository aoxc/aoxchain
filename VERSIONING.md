# AOXChain Versioning Policy

## 1. Purpose

This document defines controlled, auditable, and Git-compatible version governance.

- Current release line label: `AOXC-QTR-V1` (communication label).
- Current workspace version: `0.2.0-alpha.2`.

## 2. Canonical Version Surfaces

Version state is defined by three synchronized surfaces:

1. `Cargo.toml` (`[workspace.package].version`),
2. `configs/version-policy.toml` (`[workspace].current`),
3. Git release tags in the form `v<workspace-version>`.

All three must remain synchronized for release-grade operation.

## 3. Versioning Model

AOXChain uses a hybrid model:

- **Global workspace version**: release identity for the repository.
- **Component schema tracks**: independent schema evolution where required.

Default policy is not per-file or per-folder versioning.

## 4. SemVer Rules

### Workspace SemVer

- `MAJOR`: compatibility-breaking protocol/API/storage/governance changes,
- `MINOR`: backward-compatible capability additions,
- `PATCH`: backward-compatible fixes/hardening,
- pre-release suffixes are allowed for non-final release lines.

### Controlled-Channel Contract

`configs/version-policy.toml` must preserve:

- `strategy = "global-workspace-version-with-component-schema-tracks"`
- `release_channel = "controlled"`

These keys are release-governance controls.

## 5. Enforcement Surfaces

Version governance is enforced by:

- `tests/src/version_governance.rs`,
- `scripts/validation/versioning_gate.sh`,
- `make versioning-gate` (also in `make quality-release`).

The gate verifies synchronization, SemVer validity, tag compatibility, version-sensitive change detection, and monotonic version advancement (unless explicitly overridden).

## 6. Forced Advancement Policy

When version-sensitive surfaces change, version advancement is mandatory unless a reviewer-approved exception is documented.

Exception path:

- set `AOXC_ALLOW_NON_INCREMENTAL_VERSION=1` only for controlled maintenance flow.

## 7. Standard Workflow

1. apply implementation changes,
2. choose SemVer bump class,
3. update canonical surfaces,
4. bump schema tracks if contracts changed,
5. run `make versioning-gate` and relevant tests,
6. create/validate tag `v<workspace-version>` in release workflow.

## 8. Reviewer Checklist

Reviewers should verify:

- canonical surfaces are synchronized,
- SemVer class matches compatibility impact,
- schema bumps exist where needed,
- gates/tests are green,
- release tag plan is consistent.

## 9. Operator Guidance

If bump scope is ambiguous, prefer conservative upward bump and document rationale in PR notes.
