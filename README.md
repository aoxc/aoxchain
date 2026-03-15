# AOXChain

AOXChain is a **multi-crate Rust blockchain workspace** focused on deterministic behavior, auditability, and operational security. The repository consolidates core protocol logic, consensus, networking, API ingress, execution compatibility, and operator tooling in a single workspace.

## 1. Project Scope

The AOXChain architecture is organized across these primary domains:

- **Core protocol (`aoxcore`)**: identity, genesis, transactions, mempool, and state primitives.
- **Consensus (`aoxcunity`)**: quorum, voting, fork-choice, proposer rotation, and sealing.
- **Networking (`aoxcnet`)**: discovery, gossip, sync, and transport abstractions.
- **API ingress (`aoxcrpc`)**: HTTP + gRPC + WebSocket interfaces and security middleware.
- **Execution compatibility (`aoxcvm`)**: multi-VM/lane routing and host interfaces.
- **Operational tooling (`aoxcmd`, `aoxckit`)**: node lifecycle, economics commands, and keyforge workflows.

## 2. Quick Start

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

Local CLI validation:

```bash
cargo run -p aoxcmd -- version
cargo run -p aoxcmd -- vision
```


## 3. Production-Oriented Commands (v0.1.0-alpha Baseline)

For repeatable pre-production validation, use the quality-gate commands:

```bash
make quality-quick    # fmt + check + test
make quality          # fmt + check + clippy + test
make quality-release  # release-oriented validation
```

Additional hardening helpers:

```bash
make clippy
make audit            # requires cargo-audit installation
make package-bin
make supervise-local  # local self-healing supervisor for the node
```

The `scripts/quality_gate.sh` entrypoint is CI-friendly and supports three modes:

```bash
./scripts/quality_gate.sh quick
./scripts/quality_gate.sh full
./scripts/quality_gate.sh release
```


GitHub Actions CI runs:
- quick gate on all PRs
- full gate on pushes to protected branches

## 4. Production Readiness Note

This repository is under active development. Before production deployment, at minimum complete:

1. Independent security audits (consensus, identity, networking, RPC)
2. Threat modeling and adversarial scenario validation
3. Performance and resilience testing (stress/chaos/partition)
4. Operational runbooks, SLO/SLA targets, and observability policies
5. Release, rollback, and artifact provenance controls

## 5. Repository Map

Detailed crate index: [`crates/README.md`](crates/README.md)

| Path | Responsibility |
|---|---|
| `crates/aoxcore` | Core protocol domain primitives |
| `crates/aoxcunity` | Consensus engine |
| `crates/aoxcnet` | P2P networking layer |
| `crates/aoxcrpc` | API ingress layer |
| `crates/aoxcvm` | Execution compatibility layer |
| `crates/aoxcmd` | Node and operations command surface |
| `crates/aoxckit` | Keyforge/certificate tooling |

## 6. Documentation Policy

README files must remain synchronized with code changes. Any critical behavior update should include a README revision in the same PR.

## 7. License

MIT (`LICENSE`).
