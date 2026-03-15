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
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- interop-readiness
cargo run -p aoxcmd -- key-bootstrap --profile testnet --password "TEST#Secure2026!"
```


## 3.1 CLI Security + Telemetry Baseline

`aoxcmd key-bootstrap` now enforces a strong password baseline (minimum 12 chars with upper/lower/digit/symbol classes) before key material is persisted. On Unix-like systems, key bundle, certificate, and passport artifacts are persisted with restrictive `0600` file permissions.

`key-bootstrap` also supports `--profile mainnet|testnet`. The `testnet` profile uses `TEST-` prefixed chain/issuer defaults (for example `TEST-XXX-XX-LOCAL`) so test keys are clearly separated from mainnet-oriented defaults.

`aoxcmd runtime-status` provides a production-friendly runtime snapshot for tracing profile + Prometheus-formatted telemetry payloads and can be wired into operator dashboards or external scrape bridges.

## 3.2 Key Types (Production-Oriented Summary)

AOXChain currently uses a **post-quantum identity path** plus encrypted keyfile persistence:

- **Dilithium3** for identity signatures (`aoxcore::identity::pq_keys`)
- **Argon2id + AES-256-GCM** for secret-key encryption at rest (`aoxcore::identity::keyfile`)
- **Key bootstrap artifacts**: `<name>.key`, `<name>.cert.json`, `<name>.passport.json`

Example strong password for bootstrap:

```text
AOXc#Mainnet2026!
```

Detailed guide: [`docs/KEY_TYPES_AND_INTEROP_GUIDE_EN.md`](docs/KEY_TYPES_AND_INTEROP_GUIDE_EN.md).


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
make audit-install    # install cargo-audit
make audit            # dependency vulnerability scan
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
- weekly scheduled `cargo audit` security scan (`Security Audit` workflow)

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
