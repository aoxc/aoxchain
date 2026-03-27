# AOXC — Advanced Omnichain Execution Core

> Status: **Experimental / Under Active Construction**  
> License: **MIT**  
> Version Track: **v0.01 foundation rebuild**

AOXC is a modular, Rust-first blockchain platform targeting deterministic multi-lane execution, constitutional consensus safety, and production-ready operator workflows.

---

## 1) What We Are Building

AOXC aims to become a **high-assurance execution chain** with the following pillars:

- Deterministic execution across multiple virtual machine lanes.
- Consensus safety with explicit finality and fork-handling guarantees.
- Strong observability and operational readiness (metrics, logs, health, runbooks).
- Cross-platform developer and operator experience (Linux/macOS/Windows + Docker).

## 2) Strategic Product Thesis

AOXC is not "just another chain".
Its differentiation target is:

1. **Deterministic multi-VM settlement**
2. **Constitutional consensus governance model**
3. **Operator-first reliability posture**

## 3) Monorepo Topology

High-level modules:

- `crates/aoxcore`: transaction, block, identity, state domain logic
- `crates/aoxcunity`: consensus engine and safety/finality primitives
- `crates/aoxcvm`: multi-lane execution runtime (EVM/Move/WASM/Cardano compatibility lanes)
- `crates/aoxcnet`: networking, discovery, transport, resilience harness
- `crates/aoxcrpc`: RPC/HTTP/gRPC/websocket surfaces
- `crates/aoxcmd`: node/app bootstrap and operations entrypoints
- `tests`: integration and production-readiness style validation

## 4) Platform Compatibility (Required Baseline)

AOXC foundation is being written to run in all major environments:

- **Linux** (Ubuntu, Debian, Fedora)
- **macOS** (Apple Silicon + Intel)
- **Windows** (PowerShell, Git Bash, WSL2)
- **Docker** (local developer runtime and CI parity)

### 4.1 Rust Toolchain

- Rust stable (recommended via rustup)
- Cargo workspace support

### 4.2 Optional Tooling

- Docker / Docker Compose
- `make` (or PowerShell scripts on Windows)

## 5) Quick Start (Cross-Platform)

### 5.1 Clone + Build

```bash
git clone <repo-url> aoxchain
cd aoxchain
cargo build --workspace
```

### 5.2 Run Core Validation

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude aoxchub --all-targets --all-features -- -D warnings
cargo test --workspace --exclude aoxchub --all-targets
```

### 5.3 Desktop Surface Validation

```bash
cargo check -p aoxchub --all-targets
```

## 6) Docker-First Development

> Goal: ensure every critical command can run in a clean, reproducible container.

Planned baseline:

- Standardized dev image with Rust toolchain + common build dependencies.
- One-command workspace validation in container.
- Future: deterministic integration environment with multi-node local network.

## 7) Engineering Rules (Non-Negotiable)

1. Determinism before performance optimization.
2. Reproducible build/test in local + CI + container.
3. No merge without format/lint/tests passing.
4. Security-critical changes require threat notes and test evidence.
5. Public interfaces require compatibility notes.

## 8) Current Maturity Statement

This repository is experimental.
Breaking changes, refactors, and protocol reshaping are expected until stabilization gates are formally passed.

## 9) Primary Planning Document

See the root roadmap and execution checklist:

- [`ROADMAP.md`](./ROADMAP.md)

This file is the authoritative track for foundation, infrastructure, and release-readiness milestones.

## 10) License

MIT License.
All contributors must preserve license headers and respect third-party dependency obligations.
