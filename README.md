<div align="center">
  <a href="https://github.com/aoxc/aoxcore">
    <img src="logos/aoxc_transparent.png" alt="AOXCORE Logo" width="180" />
  </a>
</div>






## 

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
cargo run -p aoxcmd -- interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
```


## 3.1 CLI Security + Telemetry Baseline

`aoxcmd key-bootstrap` now enforces a strong password baseline (minimum 12 chars with upper/lower/digit/symbol classes) before key material is persisted. On Unix-like systems, key bundle, certificate, and passport artifacts are persisted with restrictive `0600` file permissions.

`key-bootstrap` also supports `--profile mainnet|testnet`. The `testnet` profile uses `TEST-` prefixed chain/issuer defaults (for example `TEST-XXX-XX-LOCAL`) so test keys are clearly separated from mainnet-oriented defaults.

For safety, `mainnet` profile key generation now requires explicit opt-in (`--allow-mainnet` or `AOXC_ALLOW_MAINNET_KEYS=true`) to reduce accidental production key creation during local/test runs.

`aoxcmd runtime-status` provides a production-friendly runtime snapshot for tracing profile + Prometheus-formatted telemetry payloads and can be wired into operator dashboards or external scrape bridges.

### Interop Release Gate

Use `interop-gate` for machine-readable release checks. It outputs pass/fail, readiness percentage, and missing controls, and can fail CI with `--enforce`.

Example:

```bash
cargo run -p aoxcmd -- interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
```

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

For Turkish go/no-go criteria focused on real-chain operations, see [`docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md`](docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md).

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

## 8. Mainnet/Testnet Operational Playbook (Detailed)

> ⚠️ Important: This repository provides a strong engineering baseline, but **no blockchain deployment can be guaranteed “100% error-free”**. Use staged rollout, audits, and monitored canary deployments.

### 8.1 Build + Binary Packaging

```bash
make quality-quick
make package-bin
```

### 8.2 Testnet First (Recommended)

1. Generate testnet key material:

```bash
./bin/aoxc key-bootstrap --profile testnet --password 'TEST#Secure2026!'
```

2. Bootstrap node + one block:

```bash
./bin/aoxc node-bootstrap
./bin/aoxc produce-once --tx 'testnet-smoke-1'
```

3. Verify runtime telemetry snapshot:

```bash
./bin/aoxc runtime-status --trace standard --tps 25.0 --peers 8 --error-rate 0.001
```

4. Run release gate checks:

```bash
./bin/aoxc interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
```

### 8.3 Mainnet-Oriented Bootstrap (Explicit Opt-In)

Mainnet key generation is intentionally blocked unless you explicitly allow it:

```bash
./bin/aoxc key-bootstrap --profile mainnet --allow-mainnet --password 'AOXc#Mainnet2026!'
```

Alternative env-based override:

```bash
AOXC_ALLOW_MAINNET_KEYS=true ./bin/aoxc key-bootstrap --profile mainnet --password 'AOXc#Mainnet2026!'
```

### 8.4 Continuous Block Production + Detailed Logs

Use the new script for uninterrupted block attempts and timestamped logs:

```bash
MAX_ROUNDS=0 SLEEP_SECS=2 LOG_FILE=./logs/continuous-producer.log ./scripts/continuous_producer.sh
```

- `MAX_ROUNDS=0` means infinite loop.
- Each round writes:
  - input payload (`tx=`),
  - `produce-once` output,
  - round-level status (`OK` / `ERROR code=...`).

Example finite test run:

```bash
MAX_ROUNDS=5 TX_PREFIX=testnet-batch ./scripts/continuous_producer.sh
```

### 8.5 Self-Healing Node Supervision

```bash
MAX_RESTARTS=50 RESTART_DELAY_SECS=2 ./scripts/node_supervisor.sh
```

### 8.6 End-to-End Operator Command Set

```bash
# compile + package
make quality-quick
make package-bin

# testnet bootstrap
./bin/aoxc key-bootstrap --profile testnet --password 'TEST#Secure2026!'
./bin/aoxc genesis-init --path ./configs/genesis.testnet.local.json --chain-num 2026 --block-time 2 --treasury 1000000000
./bin/aoxc node-bootstrap

# produce and observe
./bin/aoxc produce-once --tx 'initial-liquidity-seed'
./bin/aoxc runtime-status --trace verbose --tps 18.5 --peers 12 --error-rate 0.0005

# continuous producer loop with logs
MAX_ROUNDS=0 SLEEP_SECS=2 LOG_FILE=./logs/continuous-producer.log ./scripts/continuous_producer.sh
```

### 8.7 Log Review Commands

```bash
tail -n 100 ./logs/continuous-producer.log
rg "ERROR|OK|round=" ./logs/continuous-producer.log
```
