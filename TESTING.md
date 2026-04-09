# Testing and Validation Policy (Reset)

This policy defines mandatory validation for architecture-reset and post-quantum migration-safe development.

## 1) Validation Objective

Every change must preserve:

- deterministic behavior,
- fail-closed validation,
- policy-governed authority semantics,
- reproducible readiness evidence.

## 2) Mandatory Baseline Gates

Unless a stricter scope-specific control applies, run:

- `make build`
- `make test`
- `make quality`
- `make audit`
- `cargo fmt --all --check`
- `make testnet-gate`
- `make testnet-readiness-gate`

If any mandatory gate fails, readiness state is `NOT_READY`.

## 3) Required Targeted Validation for Sensitive Changes

For changes to authority, profile, replay, recovery, consensus, or serialization paths, include:

- deterministic replay tests,
- malformed-proof and policy-violation adversarial tests,
- profile mismatch/downgrade rejection tests,
- migration and recovery transition tests,
- compatibility and rollback-path checks,
- key-domain separation tests (wallet/validator/governance/recovery/transport),
- deterministic key-encoding/address-binding test vectors.

General test pass alone is insufficient for these classes.

## 4) Evidence Requirements

Validation evidence must include:

- executed command list,
- pass/fail status per gate,
- artifact references attributable to commit SHA,
- documented exceptions (if any) with risk owner.

## 5) Exception Rule

Skipped controls require explicit maintainer approval and tracked remediation owner/date.
Silent omission is prohibited.

## 6) Release Readiness Rule

A branch may be marked ready only when:

- mandatory gates pass,
- targeted sensitive-change validation is complete,
- evidence is retained and reviewable.

Otherwise status is `NOT_READY`.
