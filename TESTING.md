# Testing and Validation Policy

## Purpose

This policy defines the minimum validation expectations for all changes introduced into the AOXCHAIN workspace. Its objective is to ensure deterministic behavior, regression resistance, operational readiness, and audit-quality evidence retention across the full software lifecycle.

## Validation Layers

The project validation model is composed of the following control layers:

- **Unit Tests:** Validate crate-level logic, boundary conditions, and critical invariants in isolation.
- **Integration Tests:** Validate cross-crate behavior, subsystem interactions, and end-to-end execution paths, primarily through the `tests/` hierarchy.
- **Readiness Checks:** Validate operational and release readiness through CLI-driven control workflows, including environment assumptions, execution preconditions, and release gating checks.
- **Evidence Validation:** Validate that generated artifacts, reports, and script-produced outputs are internally consistent, reproducible where applicable, and attributable to a specific change set.

## Mandatory Baseline Commands

Unless a stricter control is explicitly required by change scope, the following commands constitute the minimum validation baseline for workspace changes:

- `cargo test --workspace --exclude aoxchub --all-targets --locked`
- `cargo clippy --workspace --exclude aoxchub --all-targets --all-features --locked -- -D warnings`
- `cargo check -p aoxchub --all-targets --locked`
- `cargo fmt --all --check`
- `make test`
- `make quality`
- `make code-size-gate` (required for changes that add or modify source files; enforces `MAX_CODE_LINES=260` by default on staged/modified files)
- `make code-size-gate-full` (recommended before major refactors to assess large-file compliance across the full tracked repository surface)
- `make os-compat-gate` (required when changes affect cross-platform runtime, Docker, or host-tooling contract)
- `make testnet-gate` (required when changes affect `configs/environments/testnet` or testnet operator flow)
- `make testnet-readiness-gate` (single-command gate for PR-ready testnet validation)

A change must not be considered validation-complete if any mandatory baseline command fails, is skipped without justification, or produces results inconsistent with the declared change scope.

## Change-Driven Validation Requirements

In addition to the baseline controls, the following change classes require targeted regression coverage and corresponding documentation updates:

- **Consensus Behavior:** Any change affecting consensus rules, message handling, validator behavior, quorum logic, or ordering semantics.
- **Deterministic Execution and State Transitions:** Any change affecting state computation, execution reproducibility, transaction ordering, or deterministic outputs.
- **Persistence Formats and Migrations:** Any change affecting stored data schemas, serialization formats, migration routines, or backward compatibility assumptions.
- **Public API Contracts:** Any change affecting externally consumed interfaces, request/response structures, CLI behavior, or documented integration surfaces.
- **Key and Signing Workflows:** Any change affecting key generation, storage, derivation, signing, verification, authorization, or operational trust boundaries.

For the above categories, general test execution alone is insufficient. The author must provide targeted regression coverage demonstrating that the modified behavior is both intentional and controlled.

## External Ingress Adversarial Validation

Changes that affect externally sourced transactions, peer admission, session establishment, or protocol-envelope verification must include adversarial regression coverage in `tests/src/external_surface_fuzz.rs`.

The external ingress suite is expected to include:

- deterministic randomized corpus checks for malformed transaction payloads and identity/signature fields;
- protocol-envelope tamper checks for chain identity, protocol serial, hash integrity, framing version, and validity-window violations;
- peer/session abuse-path checks covering duplicate admission, unknown-session use, banned-peer behavior, and malformed certificate windows.

Recommended execution command:

- `cargo test -p tests external_surface_fuzz -- --nocapture`

A change that modifies ingress validation logic without corresponding adversarial coverage updates must be treated as validation-incomplete.

## Measurable Validation Artifacts

The following repository-governed artifacts are mandatory for visibility and auditability:

- `docs/testing/TEST_MATRIX.md`: workspace test inventory snapshot generated from source markers;
- `docs/testing/COVERAGE_STATUS.md`: coverage policy and release-gate expectations;
- `docs/testing/CRITICAL_INVARIANTS.md`: protocol and operational invariants requiring regression evidence;
- `artifacts/testing/test_inventory.json`: machine-readable test inventory output.

Inventory artifacts are refreshed with:

- `make test-inventory`

When a change alters validation surfaces, the corresponding artifact set must be updated in the same change.

## Evidence and Traceability Requirements

Validation results must be traceable to the exact change under review. At minimum, validation evidence should be linked to:

- the relevant commit, branch, or release candidate identifier;
- the executed command set;
- the produced artifacts, reports, or logs retained for review;
- any documented deviations, exceptions, or known limitations.

Validation claims without retained evidence linkage must be treated as incomplete and must not be relied upon as audit-grade proof.

## Release and Review Alignment

No change should be promoted as release-ready, audit-ready, or operationally validated unless its required validation scope has been executed and its supporting evidence is reviewable.

Reviewers are expected to evaluate both:
- the correctness of the implementation; and
- the sufficiency of the validation evidence relative to the risk introduced by the change.

## Exception Handling

Any validation exception must be explicitly documented, risk-accepted by the appropriate reviewer or maintainer, and tracked for remediation where applicable. Silent omission of required validation steps is not permitted.


## Testnet Go/No-Go Decision Criteria

A branch should be treated as **testnet-ready** only when all of the following conditions are simultaneously true:

- `make testnet-gate` passes without warnings or skipped checks;
- `make testnet-readiness-gate` passes and records no skipped sub-check;
- `cargo test -p tests` passes, including testnet external readiness suites;
- required testnet bundle files under `configs/environments/testnet` remain identity-consistent and hash-consistent;
- release-policy and profile controls continue to enforce fail-closed behavior for manifest/genesis/identity checks.

If any one of these controls fails, readiness status is **not ready** and deployment should be blocked until remediation is merged and revalidated.

Recommended operator declaration format for release notes or PRs:

- `Status: TESTNET_READY` only after all gate commands pass;
- otherwise `Status: NOT_READY` with explicit failing gate and remediation owner.


## Quantum-Profile Validation Addendum

For roadmap phases affecting cryptographic profiles, the following additional validation is mandatory:

- mixed-profile deterministic consensus simulations (valid + malformed vectors);
- replay-domain separation regression tests for each active profile;
- VM (`aoxcvm`) deterministic syscall and metering regression under profile-gated execution;
- downgrade-attempt rejection tests for network/session negotiation paths;
- artifact publication for profile compatibility, performance budget, and rollback rehearsal results.

Recommended command set for profile-impacting changes:

- `make test`
- `make quality`
- `make audit`

If these checks cannot run in the current environment, the limitation must be stated explicitly with remediation plan and rerun owner.

## Phase-1 Full Determinism Closure Gate

For Phase-1 completion claims, the following integrated readiness checks are mandatory:

- `cargo test -p tests phase1_full_readiness_surface_is_consistent`
- `cargo test -p tests vm_phase1_execution_is_deterministic_across_replays`
- `cargo test -p tests block_production_is_deterministic_for_permuted_body_sections`
- `cargo test -p tests fork_choice_accepts_equal_height_siblings_with_deterministic_tiebreak`

`phase1_full_readiness_surface_is_consistent` is the umbrella regression proving that
deterministic block construction, deterministic equal-height fork-choice selection,
and deterministic AOXCVM phase-1 replay behavior hold together in one control flow.
