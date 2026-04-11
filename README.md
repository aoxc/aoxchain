<div align="center">
  <img src="https://github.com/aoxc/aoxchain/blob/main/logos/aoxc_transparent.png" alt="AOXChain Logo" width="220" />
  <h1>AOXChain</h1>
  <p><strong>Deterministic Layer-1 engineering with policy-governed authority and evidence-gated operations.</strong></p>
</div>

> **Warning — Experimental Repository**
>
> AOXChain is under active development. Interfaces, runtime behavior, policy profiles, and configuration semantics may change between commits. Do **not** infer production readiness unless a release artifact and its linked evidence bundle explicitly state readiness.

---

## 1. What This Repository Is

AOXChain is a blockchain engineering program focused on four non-negotiable properties:

1. **Determinism**: identical canonical inputs must produce identical canonical outputs.
2. **Fail-closed validation**: invalid, unsupported, or downgraded inputs are rejected before execution.
3. **Policy-governed cryptography**: signature and profile behavior is explicit, versioned, and governance controlled.
4. **Evidence-based readiness**: release or audit claims are valid only with reproducible artifacts.

In practical terms, this repository contains protocol kernels, execution engines, network and RPC surfaces, operator tooling, and governance-grade documentation required to run and assess an AOXChain environment.

## 2. What This README Covers

This document provides:

- repository purpose and structure,
- operational command surfaces (build, run, validation, chain bootstrap),
- chain lifecycle entry points,
- audit and evidence expectations,
- links to normative policy documents.

For technical invariants and normative rules, read `READ.md` after this file.

## 3. Repository Topology (Root)

Primary directories and responsibilities:

- `crates/` — Rust workspace crates (kernel, VM, networking, CLI, hub, config).
- `configs/` — environment profiles, topology metadata, compatibility policy inputs.
- `scripts/` — validation gates, operational automation, and release-evidence helpers.
- `tests/` — cross-crate and integration validation surfaces.
- `docs/` — implementation plans, runbooks, compatibility references.
- `artifacts/` — generated evidence bundles and release records.
- `models/` — machine-readable governance/readiness models.
- `contracts/` — contract-facing references and integration material.

Key root governance files:

- `READ.md` — repository technical contract and invariants.
- `ARCHITECTURE.md` — component boundaries and trust model.
- `SCOPE.md` — in/out of scope and sensitive change classes.
- `SECURITY.md` — security posture and disclosure workflow.
- `TESTING.md` — mandatory gates and evidence policy.
- `ROADMAP.md` — phased delivery and closure criteria.
- `VERSIONING.md` — release/version synchronization rules.
- `CONTRIBUTING.md` — contribution and merge-readiness discipline.

## 4. High-Level Component Map

Representative component responsibilities:

- `crates/kernel/aoxcore` — protocol primitives (identity, transactions, blocks, genesis).
- `crates/kernel/aoxcunity` — consensus kernel and safety-critical transition logic.
- `crates/aoxcvm` — deterministic VM and execution policy surfaces.
- `crates/aoxcexec` / `crates/aoxcenergy` — execution orchestration and metering.
- `crates/aoxcnet` — peer transport, admission, and network resilience.
- `crates/aoxcrpc` — RPC ingress and API control surfaces.
- `crates/aoxconfig` — typed configuration and profile composition.
- `crates/aoxcmd` — operator CLI for lifecycle and gate orchestration.
- `crates/aoxchub` — operator-facing service hub.
- `crates/aoxckit` — auxiliary operational toolkit.

## 5. Environment Prerequisites

Minimum baseline for local work:

- Rust toolchain compatible with `Cargo.toml` + `Cargo.lock`,
- POSIX shell,
- `make`,
- optional: Docker or Podman for containerized workflows.

## 6. Build and Binary Validation

### 6.1 Build core binaries

```bash
cargo build -p aoxcmd --release
cargo build -p aoxchub --release
cargo build -p aoxckit --release
```

### 6.2 Validate runtime entrypoints

```bash
cargo run -p aoxcmd --bin aoxc -- --help
cargo run -p aoxchub -- --help
cargo run -p aoxckit -- --help
make help
```

## 7. Quality and Audit Gates

Baseline command set:

```bash
make fmt
make check
make test
make clippy
make audit
make quality
```

Extended policy/release gates (use as required by scope):

```bash
make cargo-deny-gate
make code-size-gate
make versioning-gate
make repo-hygiene-gate
make production-full
make phase1-full
make quantum-readiness-gate
make aoxcvm-production-closure-gate
make quantum-full
```

Testnet readiness surfaces:

```bash
make testnet-gate
make testnet-readiness-gate
```

## 8. Operator Workflow Commands

Common operator workflows:

```bash
make demo
make localnet
make devnet
make testnet
make doctor
make audit-chain
make reset
```

Q-network helpers:

```bash
make aoxc-q-up AOXC_Q_MODE=<local|public> AOXC_Q_NODES=<n> AOXC_Q_FORCE=1
make aoxc-q-status AOXC_Q_MODE=<local|public> AOXC_Q_NODES=<n>
make aoxc-q-stop AOXC_Q_MODE=<local|public> AOXC_Q_NODES=<n>
```

## 9. Runtime Lifecycle Commands

Runtime package and activation lifecycle:

```bash
make runtime-print
make runtime-source-check
make runtime-bundle-compat-check
make runtime-install
make runtime-verify
make runtime-activate
make runtime-status
make runtime-fingerprint
make runtime-doctor
make runtime-reinstall
make runtime-reset
```

## 10. Persistent Chain Bootstrap (Detailed)

AOXChain includes make targets for persistent chain initialization and controlled bootstrap.

### 10.1 Inspect chain helper surface

```bash
make chain-help
```

### 10.2 Initialize a persistent chain

```bash
make chain-init \
  AOXC_BOOTSTRAP_PROFILE=validation \
  AOXC_VALIDATOR_NAME=validator-01 \
  AOXC_VALIDATOR_PASSWORD='StrongPass#2026'
```

### 10.3 Add an account

```bash
make chain-add-account \
  AOXC_NEW_ACCOUNT_ID=AOXC_USER_0001 \
  AOXC_NEW_ACCOUNT_BALANCE=1000000 \
  AOXC_NEW_ACCOUNT_ROLE=user
```

### 10.4 Add a validator

```bash
make chain-add-validator \
  AOXC_VALIDATOR_ID=aoxc-val-custom-001 \
  AOXC_CONSENSUS_PUBLIC_KEY=<hex> \
  AOXC_NETWORK_PUBLIC_KEY=<hex> \
  AOXC_VALIDATOR_BALANCE=50000000
```

### 10.5 Start persistent runtime

```bash
make chain-start-persistent
```

### 10.6 Chain bootstrap intent (conceptual)

1. initialize runtime and validator identity,
2. define initial accounts and balances,
3. register validator metadata and key material,
4. transition to persistent chain execution,
5. verify health and readiness using gates and audit commands.

## 11. Containerized Execution

Container surfaces:

```bash
make container-check CONTAINER_ENGINE=docker
make container-check CONTAINER_ENGINE=podman
make container-build
make container-config
make container-up
make container-down
```

For Podman-specific notes, see `PODMAN.md`.

## 12. Operations, Database, and Release Packaging

Operations commands:

```bash
make ops-help
make ops-doctor
make ops-prepare
make ops-start
make ops-once
make ops-stop
make ops-status
make ops-restart
make ops-logs
make ops-flow
```

Database and audit commands:

```bash
make db-init
make db-status
make db-event
make db-release
make db-history
make db-health
```

Packaging and release commands:

```bash
make package-bin
make package-all-bin
make package-versioned-bin
make package-versioned-archive
make publish-release
make repo-release-keygen
make repo-release-signed
make repo-release-signed-verify
make repo-release-prepare
make repo-release-validate
make repo-secure-bundle RELEASE_SIGNING_KEY=<key.pem> RELEASE_SIGNING_CERT=<cert.pem>
make repo-secure-bundle-verify RELEASE_SIGNING_CERT=<cert.pem>
make install-binaries-root
make github-install-binaries GITHUB_REPO=<owner/repo> GITHUB_VERSION=<semver>
```

## 13. Audit-Readiness and Documentation Discipline

For audit-sensitive changes, include:

- scope statement and risk classification,
- trust-boundary impact notes,
- compatibility/migration/rollback implications,
- executed command transcript,
- evidence artifacts linked to commit SHA,
- synchronized documentation updates across affected governance files.

A passing local build is **necessary** but not **sufficient** for release-readiness claims.

## 14. Security, License, and Liability

- Security policy and disclosure process: `SECURITY.md`.
- License: MIT (`LICENSE`).
- Liability posture: software is provided **"AS IS"**, without implied warranties.

## 15. Suggested Reading Order

1. `README.md` (this file) — operational orientation,
2. `READ.md` — normative technical contract,
3. `ARCHITECTURE.md` — trust boundaries and component responsibilities,
4. `SCOPE.md` and `SECURITY.md` — risk and governance constraints,
5. `TESTING.md` and `ROADMAP.md` — validation and phase closure discipline.
