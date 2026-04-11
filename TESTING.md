# AOXChain Testing and Validation Policy

This document defines mandatory validation for deterministic and migration-safe development.

## 1. Validation Objective


Every change must preserve:

- deterministic behavior,
- fail-closed validation,
- policy-governed authority semantics,
- reproducible readiness evidence.

## 2. Mandatory Baseline Gates

Unless stricter controls apply, run:

- `make build`
- `make test`
- `make quality`
- `make audit`
- `cargo fmt --all --check`
- `make testnet-gate`
- `make testnet-readiness-gate`

If any mandatory gate fails, readiness status is `NOT_READY`.

## 3. Required Targeted Validation (Sensitive Changes)

For changes touching authority, profile, replay, recovery, consensus, serialization, or key-domain logic, include:

- deterministic replay tests,
- malformed-proof and policy-violation adversarial tests,
- profile mismatch and downgrade rejection tests,
- migration and recovery transition tests,
- compatibility and rollback-path checks,
- key-domain separation tests,
- deterministic key-encoding/address-binding vectors.

General test pass is insufficient for these classes.

## 4. Evidence Requirements

Validation evidence must include:

- command list,
- pass/fail status per gate,
- artifact references attributable to commit SHA,
- explicit exceptions with risk owner.

## 5. Exception Policy

Skipped controls require explicit maintainer approval and tracked remediation owner/date.
Silent omission is prohibited.

## 6. Release Readiness Rule

A branch may be marked ready only when:

- mandatory gates pass,
- targeted sensitive-change validation is complete,
- evidence is retained and reviewable.

Otherwise readiness state is `NOT_READY`.
