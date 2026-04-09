# AOXChain Scope Statement (Reset)

This scope statement defines what the repository currently treats as authoritative work after the architecture reset.

## In Scope

- deterministic Layer-1 protocol engineering,
- policy-based authority architecture and migration-safe cryptographic agility,
- validator/governance/account authority control surfaces,
- replay-domain and recovery-domain protocol semantics,
- operational gates and evidence production for readiness declarations,
- environment configuration and release controls required for disciplined promotion.

## Out of Scope

- absolute or perpetual security guarantees,
- implicit production claims without evidence,
- undocumented compatibility promises for experimental surfaces,
- governance or operational behavior that bypasses protocol-defined validation.

## Sensitive Change Classes

The following changes require explicit design rationale, tests, and synchronized docs:

- consensus admission or finality semantics,
- authority model or policy evaluation logic,
- profile activation/deprecation and migration rules,
- replay/recovery semantics,
- serialization/storage compatibility behavior,
- validator/governance authorization paths,
- wallet/node key hierarchy, domain separation, and lifecycle controls,
- release/readiness gate definitions.

## Compatibility Policy

Compatibility is managed through explicit release policy and migration documentation.
Breaking changes may be accepted when required for determinism, safety, or architectural integrity, but they must include clear operator guidance and rollback context.

## Evidence Policy

A scope or readiness claim is authoritative only when supported by reproducible commands and retained artifacts.
Claims without evidence are non-authoritative.

## License and Liability Context

AOXChain is distributed under the MIT License on an "as is" basis, without warranties or liability assumptions by maintainers or contributors except where prohibited by applicable law.
