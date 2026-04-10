# AOXCMD Scope

## In Scope

`aoxcmd` owns the AOXChain CLI command surface and supporting orchestration logic for:

- bootstrap/config/genesis lifecycle operations,
- node runtime and network lifecycle commands,
- validator lifecycle command flows,
- query and API operator entry points,
- diagnostics, audit, and readiness evidence commands,
- command output shaping (`text/json/yaml`) and operator UX contracts.

## Out of Scope

- Consensus protocol correctness changes that belong to kernel crates.
- Network transport implementation changes that belong to networking crates.
- Governance policy definitions maintained outside CLI orchestration.
- Top-level repository policy text not specific to the `aoxcmd` command contract.

## Sensitive Change Classes

The following are treated as high-sensitivity within this crate:

- command name/subcommand changes or argument contract changes,
- machine-readable output schema changes,
- lifecycle flow semantics for `node join`, validator onboarding, and genesis gating,
- evidence generation paths and audit artifact semantics,
- any behavior that can alter operator interpretation of production readiness.

## Compatibility Expectations

- Preserve backward compatibility for existing automation-facing flags whenever possible.
- If behavior must change, expose the change explicitly in help text and release notes.
- Keep error semantics stable for invalid arguments and policy gate failures.
