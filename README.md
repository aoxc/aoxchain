# AOXChain

AOXChain is a deterministic Layer-1 engineering program focused on policy-governed authority, cryptographic agility, and evidence-gated release operations.

> **Warning — Experimental Repository**
>
> AOXChain is under active development. Interfaces, runtime behavior, policy profiles, and configuration semantics may change between commits. Do not infer production readiness unless a specific release artifact and associated evidence bundle state it explicitly.

## 1. Repository Purpose

AOXChain exists to build a blockchain runtime and operational platform that is:

- deterministic under adversarial and concurrent execution conditions,
- classically secure in current deployment profiles,
- post-quantum transition capable through explicit governance,
- auditable through synchronized architecture, scope, security, and testing controls.

No readiness assertion is considered valid without reproducible evidence.

## 2. Repository Layout

Primary top-level directories and responsibilities:

- `crates/`: Rust workspace crates for protocol, execution, networking, and operator tooling.
- `configs/`: environment profiles, topology definitions, policy metadata, and release controls.
- `docs/`: technical references, implementation blueprints, and operational playbooks.
- `scripts/`: validation, build, release, and evidence-collection automation.
- `tests/`: cross-crate and integration-level validation surfaces.
- `artifacts/`: generated evidence outputs and release-grade records.
- `models/`: machine-readable governance and readiness models.
- `contracts/`: contract-facing reference and integration surfaces.

## 3. Canonical Governance Documents

The following root documents form the repository-level engineering contract:

- `READ.md`: technical contract and non-negotiable invariants.
- `SCOPE.md`: in-scope boundaries, exclusions, and sensitive change classes.
- `ARCHITECTURE.md`: component topology, dependency direction, and trust boundaries.
- `TESTING.md`: required validation gates and readiness evidence expectations.
- `SECURITY.md`: security posture, disclosure workflow, and priority risk classes.
- `ROADMAP.md`: phased implementation plan with evidence-based closure criteria.
- `VERSIONING.md`: version-governance policy and release synchronization requirements.
- `CONTRIBUTING.md`: contribution protocol, review standards, and merge readiness expectations.

`READ.md` is intentionally named as the technical contract; it is not a typographical variant of `README.md`.

## 4. Core Components (Representative)

- `crates/aoxcmd`: operator CLI for runtime control and readiness workflows.
- `crates/aoxcvm`: VM and deterministic execution surfaces.
- `crates/kernel/aoxcore`: canonical protocol domain structures.
- `crates/kernel/aoxcunity`: consensus kernel and safety-critical transitions.
- `crates/aoxcnet`: networking, peer transport, and resilience controls.
- `crates/aoxchub`: operator-facing web and orchestration interface.
- `crates/aoxconfig`: typed configuration and profile composition surfaces.

For component-specific constraints, review the nearest crate-level `README.md`, `SCOPE.md`, and `ARCHITECTURE.md` files.

## 5. Build and Validation Quick Start

### Prerequisites

- Rust toolchain compatible with `Cargo.toml` and `Cargo.lock`.
- POSIX-compatible shell environment.
- `make` for repository-level orchestration.

### Build primary binaries

```bash
cargo build -p aoxcmd --release
cargo build -p aoxchub --release
cargo build -p aoxckit --release
```

### Baseline runtime checks

```bash
cargo run -p aoxcmd --bin aoxc -- --help
cargo run -p aoxchub -- --help
cargo run -p aoxckit -- --help
make help
```

### Baseline quality and readiness gates

```bash
make fmt
make check
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

## 6. Audit-Readiness Expectations

For audit-facing changes, contributors should include:

- explicit scope statement,
- risk and trust-boundary impact notes,
- compatibility or migration implications,
- command transcript and artifact references tied to commit SHA,
- synchronized updates to affected governance documents.

A successful local build is necessary but insufficient for release readiness.

## 7. Security and Liability Context

AOXChain is licensed under MIT and distributed on an **"AS IS"** basis, without implied warranties.

Security posture, disclosure process, and high-priority risk classes are defined in `SECURITY.md`.
