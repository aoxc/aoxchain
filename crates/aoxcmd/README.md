# AOXCMD

> Scope: `crates/aoxcmd`

## Purpose

`aoxcmd` is the AOXChain operator CLI surface. It provides deterministic,
automation-safe command entry points for bootstrap, node runtime operations,
network verification, validator lifecycle workflows, query APIs, and audit gates.

## Primary Operator Surfaces

- **Bootstrap and genesis**  
  Environment initialization, key material workflows, genesis composition and
  production-gate checks.
- **Node and network lifecycle**  
  Node bootstrap/run/status operations plus explicit join workflows:
  - `node join` (requires `--seed|--peer` and `--genesis`)
  - `network join-check` / `network join-plan` preflight checks
  - optional `--prove` attestation output for join evidence.
- **Validator lifecycle**  
  Explicit validator onboarding and management commands:
  - `validator join|register`
  - `validator activate|bond|unbond`
  - `validator set-status`
  - `validator commission-set`.
- **Observability and readiness**  
  Runtime status, diagnostics bundles, production audits, and readiness gates.

## Determinism and Output Contract

- All operator commands must preserve deterministic behavior for the same inputs.
- Output supports curated text plus machine-facing `json` and `yaml`.
- Validation and policy failures must produce explicit, stable error codes.
- Audit-oriented commands are expected to emit evidence-friendly structured data.

## Operational Notes

- Treat command argument changes as compatibility-sensitive.
- Keep lifecycle semantics explicit; avoid ambiguous aliases that hide side effects.
- For join and validator onboarding flows, prefer strict preflight checks and
  evidence generation when operating in testnet/mainnet contexts.
