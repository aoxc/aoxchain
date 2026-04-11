![AOXChain Logo](logos/aoxc_transparent.png)

# AOXChain

AOXChain is an experimental Layer-1 blockchain engineering program focused on deterministic execution, policy-governed authority, and evidence-driven operations.

> **Status Notice**
>
> This repository is under active development. Runtime behavior, command surfaces, and policy configurations can change between commits. Treat all workflows as pre-release unless an explicit release artifact and evidence bundle states otherwise.

## 1. What AOXChain Is

AOXChain is designed as a deterministic blockchain stack where protocol truth is controlled by kernel-level rules and governance-managed cryptographic policy.

At a high level, the system targets:

- deterministic state transition behavior,
- fail-closed validation before execution,
- explicit authority domains for account, validator, governance, and recovery actions,
- cryptographic agility through profile-based migration controls,
- auditability through reproducible command and artifact evidence.

## 2. Repository Structure

Top-level repository surfaces:

- `crates/` — Rust workspace crates for protocol, consensus, networking, execution, CLI, and operator tooling.
- `configs/` — environment definitions (for example mainnet/testnet profiles), runtime sources, and policy files.
- `docs/` — operational, architecture, and implementation references.
- `scripts/` — helper scripts for lifecycle operations and evidence collection.
- `tests/` — integration and cross-component test surfaces.
- `artifacts/` — generated outputs used for validation, release, and audit traces.
- `models/` — machine-readable governance and readiness metadata.
- `contracts/` — contract-facing references and integration surfaces.

## 3. Governance and Canonical Documents

The repository-wide engineering contract is defined by these root documents:

- `READ.md` — technical contract and non-negotiable invariants.
- `SCOPE.md` — in-scope vs out-of-scope boundaries and sensitive change classes.
- `ARCHITECTURE.md` — component topology, dependency direction, and trust boundaries.
- `TESTING.md` — validation requirements and readiness evidence expectations.
- `SECURITY.md` — security posture and disclosure process.
- `ROADMAP.md` — phased delivery model with closure criteria.
- `VERSIONING.md` — release and compatibility governance.
- `CONTRIBUTING.md` — contribution and review protocol.

`README.md` is the operational entrypoint. `READ.md` is the technical contract.

## 4. Chain Model (What the Chain Does)

AOXChain follows a policy-first execution model:

1. **Ingress arrives as untrusted input** through operator or service surfaces.
2. **Validation executes first** (actor identity, scheme profile, policy roots, proof bundle, replay protection).
3. **Consensus-owned truth is enforced** by kernel logic.
4. **Deterministic execution runs** only after admission succeeds.
5. **State updates and audit records are persisted** for operational and release evidence.

Normative authority pipeline:

`actor -> scheme_id -> policy_root -> proof_bundle -> replay_check -> execute`

Operationally, this means runtime correctness depends on policy and validation gates, not on ad-hoc operator overrides.

## 5. Primary Components

Representative components you will use most often:

- `aoxc` (`crates/aoxcmd`) — primary operator CLI.
- `aoxchub` (`crates/aoxchub`) — hub/service entrypoint.
- `aoxckit` (`crates/aoxckit`) — companion tooling surface.
- kernel crates under `crates/kernel/` — consensus and canonical protocol domains.

## 6. Prerequisites

Minimum local requirements:

- Rust toolchain compatible with the workspace (`Cargo.toml`/`Cargo.lock`),
- POSIX shell,
- GNU `make`.

Optional:

- Docker or Podman for container runtime flows.

## 7. Command Reference

### 7.1 Discover Available Targets

```bash
make help
```

Use this first to view the complete command catalog for your host platform and runtime root.

### 7.2 Build Commands

```bash
make build
make build-release
make build-release-all
```

- `build`: workspace build for normal development.
- `build-release`: optimized release build.
- `build-release-all`: release build across all primary binaries.

### 7.3 Code Quality and Validation Commands

```bash
make fmt
make check
make test
make clippy
make audit
make quality
```

- `fmt`: code formatting.
- `check`: compile-time and workspace checks.
- `test`: workspace tests.
- `clippy`: lint checks.
- `audit`: dependency and supply-chain audit surface.
- `quality`: aggregate quality gate.

### 7.4 Readiness and Extended Gates

```bash
make production-full
make phase1-full
make quantum-readiness-gate
make aoxcvm-production-closure-gate
make quantum-full
make testnet-gate
make testnet-readiness-gate
```

Use these when validating higher-assurance readiness, roadmap-specific closure, or testnet release posture.

### 7.5 Runtime Lifecycle Commands

```bash
make runtime-source-check
make runtime-install
make runtime-verify
make runtime-activate
make runtime-status
make runtime-doctor
```

Recommended order:

1. `runtime-source-check`
2. `runtime-install`
3. `runtime-verify`
4. `runtime-activate`
5. `runtime-status`
6. `runtime-doctor` (diagnostics)

### 7.6 Chain Bootstrap and Persistent Operations

```bash
make chain-help
make chain-init AOXC_BOOTSTRAP_PROFILE=validation AOXC_VALIDATOR_NAME=validator-01 AOXC_VALIDATOR_PASSWORD='StrongPass#2026'
make chain-add-account AOXC_NEW_ACCOUNT_ID=AOXC_USER_0001 AOXC_NEW_ACCOUNT_BALANCE=1000000 AOXC_NEW_ACCOUNT_ROLE=user
make chain-add-validator AOXC_VALIDATOR_ID=aoxc-val-custom-001 AOXC_CONSENSUS_PUBLIC_KEY=<hex> AOXC_NETWORK_PUBLIC_KEY=<hex> AOXC_VALIDATOR_BALANCE=50000000
make chain-start-persistent
```

This sequence initializes a persistent chain profile, adds identities, and starts durable chain execution.

### 7.7 Operator Workflows

```bash
make demo
make localnet
make devnet
make testnet
make doctor
make audit-chain
make reset
```

These targets provide quick-start, diagnostics, and reset paths for iterative operator workflows.

### 7.8 Packaging and Release Commands

```bash
make package-bin
make package-all-bin
make package-versioned-bin
make package-versioned-archive
make publish-release
```

For signed release workflows, use:

```bash
make repo-release-keygen
make repo-release-signed
make repo-release-signed-verify
make repo-release-prepare
make repo-release-validate
```

## 8. Minimum Readiness Baseline

A minimum readiness evaluation should include:

```bash
make build
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

If any required gate fails or is skipped without an approved exception, the system should be treated as `NOT_READY`.

## 9. Security, License, and Liability

AOXChain is distributed under the MIT License and provided on an **"AS IS"** basis, without warranty.

For security posture, disclosure expectations, and risk classes, refer to `SECURITY.md`.
