<div align="center">
  <img src="logos/aoxc_transparent.png" alt="AOXChain Logo" width="220" />

  # AOXChain
  

  **Deterministic Layer-1 engineering focused on policy-governed authority, fail-closed validation, and evidence-driven operations.**
</div>

> **Status Notice (Pre-Release)**  
> This repository is under active development. Runtime behavior, command surfaces, and governance policy may change between commits. Treat all workflows as pre-release unless an explicit release artifact and evidence bundle declares otherwise.

---

## 1) Project Overview

AOXChain is an experimental Layer-1 blockchain program built around a strict principle: **protocol truth is enforced by deterministic kernel behavior and governance-managed cryptographic policy**, not by operator discretion.

### Core Objectives

- Deterministic state transitions across nodes.
- Fail-closed validation before execution.
- Explicit authority separation across account, validator, governance, and recovery domains.
- Cryptographic agility through profile-driven migration controls.
- Reproducible evidence for audits, release readiness, and operational traceability.

---

## 2) Repository Map

Primary top-level surfaces:

- `crates/` — Rust workspace crates (protocol, consensus, execution, networking, CLI, tooling).
- `configs/` — Runtime profiles, policy inputs, and environment configuration.
- `docs/` — Architecture, operations, and implementation references.
- `scripts/` — Automation and evidence-generation helpers.
- `tests/` — Integration and cross-component validation suites.
- `artifacts/` — Generated outputs for verification and audit trails.
- `models/` — Machine-readable readiness and governance metadata.
- `contracts/` — Contract-facing references and integration surfaces.

---

## 3) Canonical Governance Documents

For engineering and review decisions, read these in order:

1. `READ.md` — Normative technical contract and invariants.
2. `SCOPE.md` — Scope boundaries, exclusions, and sensitive change classes.
3. `ARCHITECTURE.md` — Component topology, trust boundaries, and dependency direction.
4. `TESTING.md` — Validation obligations and readiness expectations.
5. `SECURITY.md` — Security posture and disclosure handling.
6. `ROADMAP.md` — Delivery phases and closure criteria.
7. `VERSIONING.md` — Compatibility and release governance.
8. `CONTRIBUTING.md` — Contribution and review protocol.

`README.md` is the operational entrypoint; `READ.md` is the normative contract.

---

## 4) Execution and Validation Model

AOXChain follows a **policy-first** admission pipeline:

1. Ingress is treated as untrusted input.
2. Validation runs before execution (actor identity, scheme profile, policy root, proof bundle, replay protection).
3. Consensus-owned truth is enforced by kernel rules.
4. Deterministic execution proceeds only after admission succeeds.
5. State updates and audit evidence are persisted.

Normative authority chain:

`actor -> scheme_id -> policy_root -> proof_bundle -> replay_check -> execute`

Operational implication: correctness depends on validation and policy gates, not ad-hoc operator override.

---

## 5) Primary Runtime Surfaces

- `aoxc` (`crates/aoxcmd`) — Primary operator CLI.
- `aoxchub` (`crates/aoxchub`) — Hub/service entrypoint.
- `aoxckit` (`crates/aoxckit`) — Companion operational toolkit.
- `crates/kernel/` — Core consensus and canonical protocol logic.

---

## 6) Prerequisites

Minimum local environment:

- Rust toolchain compatible with `Cargo.toml` / `Cargo.lock`.
- POSIX shell.
- GNU `make`.

Optional:

- Docker or Podman for containerized runtime workflows.

---

## 7) Quick Start

```bash
make help
make build
make test
make quality
```

Use `make help` first to inspect host-specific targets before selecting a workflow.

---

## 8) Command Surfaces

### 8.1 Build

```bash
make build
make build-release
make build-release-all
```

### 8.2 Quality and Validation

```bash
make fmt
make check
make test
make clippy
make audit
make quality
```

### 8.3 Readiness / Closure Gates

```bash
make production-full
make phase1-full
make quantum-readiness-gate
make aoxcvm-production-closure-gate
make quantum-full
make testnet-gate
make testnet-readiness-gate
```

### 8.4 Runtime Lifecycle

```bash
make runtime-source-check
make runtime-install
make runtime-verify
make runtime-activate
make runtime-status
make runtime-doctor
```

Recommended sequence:

1. `runtime-source-check`
2. `runtime-install`
3. `runtime-verify`
4. `runtime-activate`
5. `runtime-status`
6. `runtime-doctor`

### 8.5 Chain Bootstrap (Persistent)

```bash
make chain-help
make chain-init AOXC_BOOTSTRAP_PROFILE=validation AOXC_VALIDATOR_NAME=validator-01 AOXC_VALIDATOR_PASSWORD='StrongPass#2026'
make chain-add-account AOXC_NEW_ACCOUNT_ID=AOXC_USER_0001 AOXC_NEW_ACCOUNT_BALANCE=1000000 AOXC_NEW_ACCOUNT_ROLE=user
make chain-add-validator AOXC_VALIDATOR_ID=aoxc-val-custom-001 AOXC_CONSENSUS_PUBLIC_KEY=<hex> AOXC_NETWORK_PUBLIC_KEY=<hex> AOXC_VALIDATOR_BALANCE=50000000
make chain-start-persistent
```

### 8.6 Operator Workflows

```bash
make demo
make localnet
make devnet
make testnet
make doctor
make audit-chain
make reset
```

### 8.7 Packaging and Release

```bash
make package-bin
make package-all-bin
make package-versioned-bin
make package-versioned-archive
make publish-release
```

Signed release support:

```bash
make repo-release-keygen
make repo-release-signed
make repo-release-signed-verify
make repo-release-prepare
make repo-release-validate
```

---

## 9) Minimum Readiness Baseline

A minimal readiness evaluation should include:

```bash
make build
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

If required gates fail, or are skipped without an approved exception, treat the system as **`NOT_READY`**.

---

## 10) Security, License, and Liability

AOXChain is distributed under the MIT License and provided on an **"AS IS"** basis, without warranty.

For security classes, disclosure process, and operational posture, see `SECURITY.md`.
