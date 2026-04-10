# AOXCMD Architecture

## Component Model

`aoxcmd` is structured as a routed command plane:

1. **CLI router layer (`src/cli/mod.rs`)**  
   Parses top-level group/subcommand intent and dispatches into domain handlers.
2. **Domain operation layers (`src/cli/ops/*`, `src/cli/bootstrap/*`, `src/cli/audit/*`)**  
   Perform argument validation, lifecycle orchestration, and output rendering.
3. **Runtime/service integrations**  
   Bridge to node lifecycle, ledger operations, config/state loaders, and evidence paths.

## Control Flow and Validation

- Arguments are parsed and normalized at command entry.
- Required flags and range constraints are enforced before state mutation.
- Handler output is serialized through a unified output-format layer.
- Policy/readiness failures return explicit application error codes.

## Lifecycle Surfaces

### Node / Network Join

- `node join` enforces deterministic onboarding prerequisites (`--seed|--peer`, `--genesis`).
- `network join-check` / `network join-plan` run preflight checks and report pass/warn/fail signals.
- `--prove` produces an attestation artifact for auditability.

### Validator Lifecycle

- `validator join|register` capture onboarding intent and metadata.
- `validator activate|bond|unbond` map to stake lifecycle transitions.
- `validator set-status` and `validator commission-set` enforce explicit status/range validation.

## Trust and Boundary Notes

- CLI is an orchestration surface, not canonical protocol truth.
- Consensus-critical validation remains in kernel crates.
- Any command that writes artifacts or mutates local runtime state must keep
  file paths, preconditions, and failure modes explicit.
