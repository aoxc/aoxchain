# AOXCHub Scope

This document defines the authoritative engineering scope for the `aoxchub` crate.

## In Scope

- Provide a localhost-only operator control plane for AOXChain.
- Execute approved AOXC and Make workflows through an immutable command catalog.
- Require explicit operator confirmation with deterministic command preview before execution.
- Enforce environment-specific policy constraints for MAINNET and TESTNET workflows.
- Apply bounded runtime controls for concurrency, timeout, and output retention.

## Out of Scope

- Reimplement AOXC protocol or consensus behavior.
- Provide remote, multi-tenant, or internet-exposed control-plane operation.
- Own canonical kernel responsibilities such as chain state, consensus, or key authority.
- Execute arbitrary free-form command text without catalog policy validation.

## Sensitive Change Classes

The following change classes are considered high risk and require synchronized tests and documentation updates:

- Command catalog entries, resolution logic, or execution semantics.
- Binary source trust policy, especially MAINNET source restrictions.
- Runner capacity limits, timeout behavior, and output bounding controls.
- Loopback security gate, HTTP/SSE boundary behavior, or local-origin enforcement.
- Operator confirmation and execution authorization flow between UI and backend.

## Compatibility Expectations

- API and operator workflow behavior should not change implicitly; intentional breaks must be explicitly documented.
- Command preview strings must remain semantically equivalent to the executed program path and arguments.
- Default security posture must remain fail-closed under policy or validation uncertainty.

## Validation Expectations

- AOXCHub is an orchestration and operator-safety surface; reliability claims require reproducible evidence.
- Changes should be validated with crate-level tests and relevant repository quality gates.
- MAINNET/TESTNET policy boundary regressions are release-blocking defects.

## License and Liability Context

AOXCHub is distributed under the AOXChain MIT license on an "as is" basis; operators remain responsible for host hardening and change control.
