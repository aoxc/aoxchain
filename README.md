# AOXChain

AOXChain is a deterministic Layer-1 engineering program designed for policy-governed authority, cryptographic agility, and evidence-gated release discipline.

> **Warning — Experimental Repository**
>
> AOXChain is under active development and should be treated as an **experimental system**. Interfaces, runtime behavior, network policy, and configuration formats may change between commits. Do not assume production safety unless a specific release artifact and readiness evidence explicitly state otherwise.

## 1) Project Purpose

AOXChain exists to build a blockchain runtime and operations surface that is:

- deterministic under adversarial execution,
- classical-secure in current deployments,
- post-quantum transition-capable under governance,
- auditable through explicit architecture, scope, testing, and release evidence.

The repository emphasizes engineering discipline over marketing claims: no readiness statement is valid without reproducible evidence.

## 2) Repository Layout

Top-level directories and their operational role:

- `crates/` — Rust workspace crates for runtime, kernel, networking, tooling, and operator surfaces.
- `configs/` — environment profiles, topology definitions, registry data, and release/network metadata.
- `docs/` — program plans, runbooks, matrixes, and deep technical references.
- `scripts/` — automation for validation gates, environment checks, runtime orchestration, and release evidence.
- `tests/` — cross-crate and external-readiness integration test surfaces.
- `artifacts/` — generated evidence bundles, closure snapshots, and release manifests.
- `models/` — machine-readable readiness and governance models.
- `contracts/` — contract/system-side reference material and integration surfaces.

## 3) Canonical Governance and Technical Documents

The following repository-root documents are the primary governance and engineering contract:

- `READ.md` — operational technical contract and invariants.
- `SCOPE.md` — in-scope and out-of-scope boundaries; sensitive change classes.
- `ARCHITECTURE.md` — component structure, trust boundaries, and dependency direction.
- `TESTING.md` — mandatory validation surfaces and release-readiness expectations.
- `SECURITY.md` — security posture, disclosure process, and priority classes.
- `ROADMAP.md` — phased execution path and closure checkpoints.
- `VERSIONING.md` — compatibility and versioning posture.
- `CONTRIBUTING.md` — contribution workflow and review expectations.

When implementation changes affect architecture, compatibility, or security posture, update the corresponding governance document in the same change set.

## 4) Core Components

High-level component map (non-exhaustive):

- `crates/aoxcmd` — operator CLI for chain/runtime operations and readiness tooling.
- `crates/aoxcvm` — virtual machine, execution model, object system, and governance-enforced policy surfaces.
- `crates/kernel/aoxcore` — core protocol/domain structures (identity, transactions, blocks, genesis).
- `crates/kernel/aoxcunity` — consensus kernel and safety-critical state transitions.
- `crates/aoxcnet` — networking, gossip, p2p transport, and resilience helpers.
- `crates/aoxchub` — operator-facing web hub and command execution surface.
- `crates/aoxconfig` — configuration model and profile composition.

For crate-specific scope and architecture constraints, consult each crate’s `README.md`, `SCOPE.md`, and `ARCHITECTURE.md` when present.

## 5) Build and Validation Quick Start

### Prerequisites

- Rust toolchain compatible with `Cargo.toml` and lockfile.
- Standard UNIX shell environment.
- `make` for repository-level gate execution.

### Build key operator binary

```bash
cargo build -p aoxcmd --release
cargo build -p aoxchub --release
cargo build -p aoxckit --release
```

### Baseline repository gates

```bash
cargo run -p aoxcmd --bin aoxc -- --help
cargo run -p aoxchub -- --help
cargo run -p aoxckit -- --help
make help
```

### 5.4 Baseline quality and readiness gates

```bash
make fmt
make check
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

These gates are intended to provide a minimum confidence baseline; specific release decisions require the full testing and evidence criteria documented in `TESTING.md` and release scripts.

## 6) Configuration and Environment Profiles

Environment and topology materials are maintained under `configs/`, including:

- network profiles (`devnet`, `testnet`, `mainnet`),
- topology policies,
- registry and compatibility manifests,
- environment-specific metadata and certificates.

Treat these files as operationally sensitive. Changes may alter consensus behavior, node coordination, interoperability expectations, or release validity.

## 7) Testing and Readiness Model

AOXChain uses layered validation, including:

- unit/integration tests at crate level,
- cross-surface readiness tests under `tests/`,
- scripted quality and policy gates under `scripts/validation/`,
- artifact-based release evidence under `artifacts/` and `releases/`.

A "green local build" alone does not imply production readiness.

## 8) Security and Risk Posture

Security and trust boundaries are defined by repository governance documents and crate-level security/architecture files.

Operational expectations:

- follow responsible disclosure guidance in `SECURITY.md`,
- avoid introducing undocumented trust-boundary shifts,
- include compatibility/risk rationale for sensitive changes,
- keep auditability and deterministic behavior explicit in reviews.

## 9) Compatibility and Change Control

Compatibility-sensitive domains include (non-exhaustive):

- authority and identity model,
- consensus semantics,
- transaction/execution determinism,
- storage and object lifecycle behavior,
- network/profile and policy-governed feature activation.

If your change impacts one of these domains, update architecture/scope/testing artifacts and include explicit migration or rollback implications.

## 10) Contributing

1. Read `CONTRIBUTING.md`, `SCOPE.md`, and `ARCHITECTURE.md` before making non-trivial changes.
2. Keep changes minimal and intentional; avoid broad incidental edits.
3. Run relevant tests and validation gates before proposing a merge.
4. Include evidence and rationale for policy-sensitive or compatibility-sensitive modifications.

## 11) License and Liability

AOXChain is distributed under the MIT License.

Unless required by applicable law or explicitly agreed in writing, the software is provided **"AS IS"**, without warranties or conditions of any kind, and without maintainers assuming liability for operational outcomes.

---

For additional implementation details, begin with `READ.md`, then follow crate-level documentation nearest to the component you are modifying.
