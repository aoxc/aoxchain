# AOXChain Repository Production Gap Report

_Last reviewed: 2026-04-07 (UTC)_

## Purpose

This report captures repository-level production-readiness gaps found through deterministic local validation and governance-document review. It is intended to make current limitations explicit and operationally actionable.

## Validation Snapshot

The following checks were executed on the workspace during this review window:

- `cargo check --workspace --locked`
- `cargo test --workspace --exclude aoxchub --all-targets --locked`
- `cargo audit` (operator execution evidence from Arch Linux, 2026-04-07)

Result summary:

- compile surface: pass,
- test surface (non-`aoxchub` workspace): pass,
- no failing Rust tests detected in the executed scope,
- dependency audit reported one allowed warning: `RUSTSEC-2024-0380` for `pqcrypto-dilithium` (`0.5.0`) as unmaintained.

Observed security advisory detail:

- advisory id: `RUSTSEC-2024-0380`
- crate: `pqcrypto-dilithium`
- replacement guidance: `pqcrypto-mldsa`
- affected internal roots observed in dependency tree: `aoxcunity`, `aoxcore`, and transitively multiple workspace crates.

## Confirmed Strengths

1. Workspace crates compile successfully under locked dependency resolution.
2. Determinism and readiness suites under `tests/` pass in the executed profile.
3. Repository governance surfaces (`README.md`, `READ.md`, `SCOPE.md`, `ARCHITECTURE.md`, `SECURITY.md`, `TESTING.md`) exist and are structurally aligned.

## Open Gaps Before Production Claim

The items below are not defects in code correctness from this run, but they are gating items for strict production assertions:

1. **Cryptography dependency advisory closure is pending**
   - `cargo audit` reports `RUSTSEC-2024-0380` for `pqcrypto-dilithium` across consensus-relevant dependency paths.
   - Audit-ready status for production promotion remains blocked until migration (`pqcrypto-mldsa`) or explicit governance risk acceptance is completed and documented.

2. **Gate completeness evidence not yet attached to this review**
   - `TESTING.md` defines additional mandatory gates (`make quality`, `make audit`, testnet identity gates, and others) that were not fully re-executed in this pass.
   - Production-grade declaration should remain blocked until full gate evidence is retained for the target commit.

3. **`aoxchub` test execution split remains an explicit operational consideration**
   - The baseline Rust test command intentionally excludes `aoxchub`.
   - Release/readiness records should explicitly document this split and include separate validation evidence for hub-related surfaces when they are in scope.

4. **Readiness evidence packaging discipline must remain strict**
   - For promotion candidates, command outputs, artifact hashes, and gate status should be archived together to preserve auditability and reproducibility.

## Recommended Closure Actions

1. Open a tracked migration work item from `pqcrypto-dilithium` to `pqcrypto-mldsa` for `aoxcore` and `aoxcunity`, including compatibility and determinism tests.
2. If migration cannot be immediate, create explicit temporary risk acceptance with expiry date and compensating controls.
3. Run and record the full gate bundle declared in `TESTING.md` for the candidate branch.
4. Include environment-specific gate records (`testnet` / `mainnet`) when relevant to the release track.
5. Attach evidence references in release notes or PR description using immutable artifact paths and commit identifiers.

## Decision Rule

Status classification for this review window:

- **Code health (executed scope):** `PASS`
- **Production claim status:** `CONDITIONALLY BLOCKED` until full gate evidence set is attached for the release candidate.
