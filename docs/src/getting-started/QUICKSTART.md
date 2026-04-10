# Quickstart

This quickstart is the shortest safe path to run AOXChain locally, validate health, and inspect readiness evidence.

## 1) Prerequisites

- Rust toolchain compatible with the workspace `Cargo.toml`.
- `git`, `make`, and a POSIX shell.
- Optional: container runtime if you use containerized workflows.

## 2) Build the workspace

From repository root:

```bash
cargo build --workspace
```

Expected result: all crates compile successfully without modifying lockfiles.

## 3) Run baseline tests

```bash
cargo test --workspace
```

For faster first-pass validation, start from targeted suites in `TESTING.md`, then escalate to full matrix.

## 4) Start operator docs

Render and serve docs locally:

```bash
mdbook serve docs
```

Open the local URL and follow this order:

1. `Overview -> System Status and Operational Posture`
2. `Operations -> Full Node Guide`
3. `Testing -> Test Matrix`

## 5) Production readiness checks

Use documented gate scripts from `scripts/validation/` and readiness reports under `artifacts/`.

A practical sequence:

1. repository hygiene gate,
2. compatibility and identity gates,
3. testnet and production closure gates.

## 6) Before shipping changes

- Verify impacted architecture/security/testing docs are updated in the same change.
- Keep compatibility-sensitive behavior explicitly documented.
- Include evidence artifacts when a policy requires them.
