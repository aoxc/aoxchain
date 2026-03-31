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

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --all --check`
- `make test`
- `make quality`
- `make testnet-gate` (required when changes affect `configs/environments/testnet` or testnet operator flow)

A change must not be considered validation-complete if any mandatory baseline command fails, is skipped without justification, or produces results inconsistent with the declared change scope.

## Change-Driven Validation Requirements

In addition to the baseline controls, the following change classes require targeted regression coverage and corresponding documentation updates:

- **Consensus Behavior:** Any change affecting consensus rules, message handling, validator behavior, quorum logic, or ordering semantics.
- **Deterministic Execution and State Transitions:** Any change affecting state computation, execution reproducibility, transaction ordering, or deterministic outputs.
- **Persistence Formats and Migrations:** Any change affecting stored data schemas, serialization formats, migration routines, or backward compatibility assumptions.
- **Public API Contracts:** Any change affecting externally consumed interfaces, request/response structures, CLI behavior, or documented integration surfaces.
- **Key and Signing Workflows:** Any change affecting key generation, storage, derivation, signing, verification, authorization, or operational trust boundaries.

For the above categories, general test execution alone is insufficient. The author must provide targeted regression coverage demonstrating that the modified behavior is both intentional and controlled.

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
