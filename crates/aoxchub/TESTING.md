# AOXCHUB Testing Strategy

## Test Scope

AOXCHUB validation focuses on deterministic read-model behavior, network-profile
resolution, and UI-safe integration logic used by the desktop control plane.

## Required Checks

- Unit tests for pure service logic (for example profile resolution and
  integration grading).
- Workspace-level compilation checks for default and expanded feature sets.
- Clippy linting for safety and maintainability regressions.

## Commands

- `cargo test -p aoxchub`
- `cargo test -p aoxchub --all-features`
- `cargo clippy -p aoxchub --all-targets -- -D warnings`

## Operational Validation Notes

- Any behavior that changes profile selection, telemetry interpretation, or
  governance/wallet control messaging must include focused regression tests.
- Security-sensitive UI claims must map to explicit service evidence strings and
  fail safely when authoritative data is unavailable.
- This project is MIT licensed and provided without warranty; tests reduce
  operational risk but do not constitute a security guarantee.
