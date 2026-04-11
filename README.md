<div align="center">
  <img src="logos/aoxc_transparent.png" alt="AOXChain Logo" width="220" />

  # AOXChain

  **Deterministic Layer-1 Engineering Program**  
  Policy-governed authority • Cryptographic agility • Evidence-driven operations

  ![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
  ![Status: Experimental](https://img.shields.io/badge/Status-Experimental-orange)
  ![Rust](https://img.shields.io/badge/Stack-Rust-informational)
</div>

---

> [!WARNING]
> **Experimental Repository**  
> AOXChain is under active development. Interfaces, runtime behavior, policy profiles, and command surfaces may change between commits. Do not infer production readiness unless a specific release artifact and evidence bundle explicitly states it.

## 📘 Overview

AOXChain is a policy-first, deterministic blockchain stack designed for auditable, migration-safe operation.

Core engineering goals:

- deterministic state transitions under adversarial conditions,
- fail-closed validation before execution,
- explicit authority domains and policy roots,
- cryptographic profile agility (including migration windows),
- evidence-backed readiness assertions.

---

## 🧭 Quick Navigation

- [Repository Structure](#-repository-structure)
- [Governance and Contract Documents](#-governance-and-contract-documents)
- [Chain Execution Model](#-chain-execution-model)
- [Primary Components](#-primary-components)
- [Prerequisites](#-prerequisites)
- [Command Catalog](#-command-catalog)
- [Minimum Readiness Baseline](#-minimum-readiness-baseline)
- [Security, License, and Liability](#-security-license-and-liability)

---

## 🗂 Repository Structure

| Path | Purpose |
|---|---|
| `crates/` | Rust workspace crates for protocol, consensus, execution, networking, CLI, and operator tooling. |
| `configs/` | Environment profiles, runtime source files, topology metadata, and policy configuration. |
| `docs/` | Architecture, operations, and technical references. |
| `scripts/` | Operational automation and evidence collection helpers. |
| `tests/` | Integration and cross-component validation surfaces. |
| `artifacts/` | Generated evidence outputs and release-grade records. |
| `models/` | Machine-readable governance and readiness models. |
| `contracts/` | Contract-facing references and integration surfaces. |

---

## 🏛 Governance and Contract Documents

The repository-level engineering contract is anchored by:

- `READ.md` — technical contract and non-negotiable invariants,
- `SCOPE.md` — in-scope boundaries and sensitive change classes,
- `ARCHITECTURE.md` — component topology and trust boundaries,
- `TESTING.md` — validation and release-readiness expectations,
- `SECURITY.md` — security posture and disclosure process,
- `ROADMAP.md` — phased execution and closure criteria,
- `VERSIONING.md` — release and compatibility governance,
- `CONTRIBUTING.md` — contribution protocol and review discipline.

`README.md` is the operational entrypoint; `READ.md` is the technical contract.

---

## ⛓ Chain Execution Model

AOXChain follows a strict policy-first pipeline:

1. untrusted ingress enters via operator/service surfaces,
2. validation executes before any state transition,
3. kernel-owned consensus truth decides admission,
4. deterministic execution runs only after admission,
5. state and evidence records are persisted.

Normative authority sequence:

```text
actor -> scheme_id -> policy_root -> proof_bundle -> replay_check -> execute
```

This model prevents ad-hoc operational override of consensus-relevant truth.

---

## 🧩 Primary Components

- **`aoxc`** (`crates/aoxcmd`) — primary operator CLI.
- **`aoxchub`** (`crates/aoxchub`) — hub/service orchestration surface.
- **`aoxckit`** (`crates/aoxckit`) — companion toolkit.
- **Kernel crates** (`crates/kernel/*`) — canonical consensus and protocol domain ownership.

---

## ⚙️ Prerequisites

Required:

- Rust toolchain compatible with `Cargo.toml` and `Cargo.lock`,
- POSIX shell,
- GNU `make`.

Optional:

- Docker or Podman for containerized workflows.

---

## 🛠 Command Catalog

### 1) Discover Available Targets

```bash
make help
```

Use this first to inspect all supported targets for your host and runtime profile.

### 2) Build

```bash
make build
make build-release
make build-release-all
```

### 3) Code Quality and Validation

```bash
make fmt
make check
make test
make clippy
make audit
make quality
```

### 4) Readiness and Extended Assurance Gates

```bash
make production-full
make phase1-full
make quantum-readiness-gate
make aoxcvm-production-closure-gate
make quantum-full
make testnet-gate
make testnet-readiness-gate
```

### 5) Runtime Lifecycle

```bash
make runtime-source-check
make runtime-install
make runtime-verify
make runtime-activate
make runtime-status
make runtime-doctor
```

Recommended operational order:

1. `runtime-source-check`
2. `runtime-install`
3. `runtime-verify`
4. `runtime-activate`
5. `runtime-status`
6. `runtime-doctor`

### 6) Persistent Chain Bootstrap

```bash
make chain-help
make chain-init AOXC_BOOTSTRAP_PROFILE=validation AOXC_VALIDATOR_NAME=validator-01 AOXC_VALIDATOR_PASSWORD='StrongPass#2026'
make chain-add-account AOXC_NEW_ACCOUNT_ID=AOXC_USER_0001 AOXC_NEW_ACCOUNT_BALANCE=1000000 AOXC_NEW_ACCOUNT_ROLE=user
make chain-add-validator AOXC_VALIDATOR_ID=aoxc-val-custom-001 AOXC_CONSENSUS_PUBLIC_KEY=<hex> AOXC_NETWORK_PUBLIC_KEY=<hex> AOXC_VALIDATOR_BALANCE=50000000
make chain-start-persistent
```

### 7) Operator Workflows

```bash
make demo
make localnet
make devnet
make testnet
make doctor
make audit-chain
make reset
```

### 8) Packaging and Release

```bash
make package-bin
make package-all-bin
make package-versioned-bin
make package-versioned-archive
make publish-release
```

Signed release workflow:

```bash
make repo-release-keygen
make repo-release-signed
make repo-release-signed-verify
make repo-release-prepare
make repo-release-validate
```

---

## ✅ Minimum Readiness Baseline

A minimum readiness claim should include the following gates:

```bash
make build
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

If required gates fail (or are skipped without approved exception), readiness state should be treated as `NOT_READY`.

---

## 🔐 Security, License, and Liability

AOXChain is distributed under the MIT License and provided on an **"AS IS"** basis, without warranty.

Security posture, disclosure workflow, and high-priority risk classes are documented in `SECURITY.md`.
