# Testing

## Test Types
- Unit tests in crate-level `src/` modules.
- Integration tests in crate `tests/` folders and top-level `tests/` workspace.
- Deterministic environment and readiness validation through scripts and artifacts.

## Critical Flows
- State transition correctness.
- Consensus and networking resilience.
- RPC correctness and backward-compatible behavior.
- Configuration validation for target environments.

## Commands
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -D warnings`
- `cargo fmt --all --check`

## Notes
Any change affecting consensus, execution determinism, persisted state formats, or public API contracts must include targeted regression coverage.
