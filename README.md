<div align="center">
  <a href="https://github.com/aoxc/aoxcore">
    <img src="logos/aoxc_transparent.png" alt="AOXChain Logo" width="180" />
  </a>

# AOXChain
### Deterministic, Security-Oriented, Multi-Crate Blockchain Workspace

</div>

AOXChain is a Rust workspace for building and operating a deterministic coordination chain with strong operator controls, auditable runtime behavior, and production-readiness gates.

This README is intentionally written as an **operator-first, chronological guide** so teams can move from local bootstrap to multi-role validation with minimal ambiguity.

---

## Table of Contents

1. [What AOXChain Contains](#1-what-aoxchain-contains)
2. [Node Roles (Validator / DAO / AI Security)](#2-node-roles-validator--dao--ai-security)
3. [Prerequisites](#3-prerequisites)
4. [Build, Quality Gates, and Verification](#4-build-quality-gates-and-verification)
5. [Chronological Setup: Genesis → Node → Wallet-like Identity → Stake](#5-chronological-setup-genesis--node--wallet-like-identity--stake)
6. [Networking and Node Connectivity (Ports, IDs, Peer Prep)](#6-networking-and-node-connectivity-ports-ids-peer-prep)
7. [Staking and Economy Operations](#7-staking-and-economy-operations)
8. [Production-Oriented Commands](#8-production-oriented-commands)
9. [Make Targets for Daily and Release Operations](#9-make-targets-for-daily-and-release-operations)
10. [Security Baselines and Mainnet Safeguards](#10-security-baselines-and-mainnet-safeguards)
11. [Real-Network Readiness (Go/No-Go)](#11-real-network-readiness-gono-go)
12. [Complete `aoxcmd` Command Surface](#12-complete-aoxcmd-command-surface)

---

## 1) What AOXChain Contains

AOXChain is organized as a modular Rust workspace:

| Layer | Crate(s) | Responsibility |
|---|---|---|
| Protocol Core | `aoxcore` | Identity, genesis, transactions, mempool, protocol primitives |
| Consensus | `aoxcunity` | Quorum, rounds, voting, proposer/finality-related state |
| Networking | `aoxcnet` | Discovery, gossip, sync, transport probes |
| API Ingress | `aoxcrpc` | HTTP / gRPC / WebSocket surfaces |
| Execution Compatibility | `aoxcvm` | Multi-lane compatibility (EVM/WASM/Move/UTXO-facing paths) |
| Operator Tooling | `aoxcmd`, `aoxckit` | Node lifecycle, bootstrap, economics, readiness and audit commands |

For crate-level map details, see [`crates/README.md`](crates/README.md).

---

## 2) Node Roles (Validator / DAO / AI Security)

AOXChain operations can be modeled with three role types:

### A) Validator Node
- Produces and validates blocks.
- Maintains chain liveness and consensus participation.
- Typically runs long-lived node loops and uptime-focused supervision.

### B) DAO Governance Node
- Focuses on governance process integrity.
- Participates in voting, policy checks, operational review, and governance audits.
- Tracks release gates, audit results, and decision logs.

### C) AI Security Node
- Provides model-governed security contributions.
- Should run only under explicit policy controls (signed model, prompt guard, anomaly checks, human override).
- Can be integrated into reward logic when its security contribution is measurable and policy-compliant.

> Note: role economics/governance reward policy should be enforced by governance specs and release policies, not by README text alone.

---

## 3) Prerequisites

- Rust stable toolchain
- `cargo`
- Linux/macOS shell environment recommended for ops scripts

Optional but useful:
- `make`
- `cargo-audit`

---

## 4) Build, Quality Gates, and Verification

Baseline workspace checks:
# AOXChain

AOXChain is a **multi-crate Rust blockchain workspace** focused on deterministic behavior, auditability, and operator safety.

This repository contains:
- protocol primitives,
- consensus and networking layers,
- API ingress,
- execution compatibility surfaces,
- operational CLI tooling.

---

## 1) Architecture at a glance

| Domain | Crate(s) | Responsibility |
|---|---|---|
| Core protocol | `aoxcore` | Identity, genesis, transactions, mempool, shared primitives |
| Consensus | `aoxcunity` | Quorum, rounds, vote handling, finalization-related state |
| Networking | `aoxcnet` | Discovery/gossip/sync and transport utilities |
| API ingress | `aoxcrpc` | HTTP/gRPC/WebSocket service entry surfaces |
| Execution compatibility | `aoxcvm` | Lane-based compatibility (EVM/WASM/Move/UTXO-facing adapters) |
| Operator tooling | `aoxcmd`, `aoxckit` | Node bootstrap, runtime commands, key and ops workflows |

Detailed crate map: [`crates/README.md`](crates/README.md)

---

## 2) Quick start (local)

### Prerequisites
- Rust toolchain (stable)
- `cargo`

### Workspace validation

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

Fast operator sanity checks:
### Basic CLI sanity checks

```bash
cargo run -p aoxcmd -- version
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- port-map
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- node-run --rounds 5 --sleep-ms 1000 --tx-prefix AOXC_RUN
cargo run -p aoxcmd -- real-network --rounds 5 --timeout-ms 3000 --pause-ms 250
cargo run -p aoxcmd -- interop-readiness
cargo run -p aoxcmd -- key-bootstrap --profile testnet --password "TEST#Secure2026!"
cargo run -p aoxcmd -- interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
```

---

## 5) Chronological Setup: Genesis → Node → Wallet-like Identity → Stake

This is the recommended order for first-time environment setup.

### Step 1 — Isolate runtime data directory

```bash
export AOXC_HOME=$PWD/.aoxc-local
```

### Step 2 — Create node identity / wallet-like key material

```bash
cargo run -p aoxcmd -- key-bootstrap \
  --profile testnet \
  --name validator-01 \
  --password "TEST#Secure2026!"
```

### Step 3 — Initialize genesis

```bash
cargo run -p aoxcmd -- genesis-init \
  --chain-num 1001 \
  --block-time 6 \
  --treasury 1000000000000
```

### Step 4 — Bootstrap node runtime

```bash
cargo run -p aoxcmd -- node-bootstrap
```

### Step 5 — Produce one block (deterministic smoke)

```bash
cargo run -p aoxcmd -- produce-once --tx "boot-sequence-1"
```

### Step 6 — Run continuous local production loop

```bash
cargo run -p aoxcmd -- node-run --rounds 20 --sleep-ms 1000 --tx-prefix AOXC_RUN
```

### Step 7 — Run repeated live TCP probe

```bash
cargo run -p aoxcmd -- real-network \
  --rounds 10 \
  --timeout-ms 3000 \
  --pause-ms 250 \
  --bind-host 127.0.0.1 \
  --port 0
```

### Step 8 — Runtime and release readiness checks
---

## 3) Most useful operator commands

> All commands below support `--home <dir>` globally (or `AOXC_HOME`) for data isolation.

### 3.1 Bootstrap and first block

```bash
cargo run -p aoxcmd -- key-bootstrap --profile testnet --password "TEST#Secure2026!"
cargo run -p aoxcmd -- genesis-init
cargo run -p aoxcmd -- node-bootstrap
cargo run -p aoxcmd -- produce-once --tx "hello-aox"
```

### 3.2 Runtime snapshot and release gate

```bash
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- interop-readiness
cargo run -p aoxcmd -- interop-gate \
  --audit-complete true \
  --fuzz-complete true \
  --replay-complete true \
  --finality-matrix-complete true \
  --slo-complete true \
  --enforce
cargo run -p aoxcmd -- interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
./scripts/quality_gate.sh quick
./scripts/quality_gate.sh full
./scripts/quality_gate.sh release
```

---

## 6) Networking and Node Connectivity (Ports, IDs, Peer Prep)

Operational checklist for node connectivity:

1. Generate identity material per node (`key-bootstrap`).
2. Confirm ports and surfaces with `port-map`.
3. Capture node identity metadata from generated artifacts (`*.cert.json`, `*.passport.json`, keys).
4. Build a peer list and verify reachable addresses across distinct hosts.
5. Validate smoke path (`network-smoke`) and multi-round transport behavior (`real-network`).
For Turkish go/no-go criteria focused on real-chain operations, see [`docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md`](docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md).

1. Independent security audits (consensus, identity, networking, RPC)
2. Threat modeling and adversarial scenario validation
3. Performance and resilience testing (stress/chaos/partition)
4. Operational runbooks, SLO/SLA targets, and observability policies
5. Release, rollback, and artifact provenance controls

Inspect configured ports:

```bash
cargo run -p aoxcmd -- port-map
```

Run explicit TCP smoke probe:

```bash
cargo run -p aoxcmd -- network-smoke --bind-host 127.0.0.1 --port 9600 --payload "HELLO"
```

> Production note: loopback tests are not sufficient. Validate multi-host, firewall-aware, latency/packet-disturbance scenarios before claiming real-network readiness.

---

## 7) Staking and Economy Operations

Initialize economy state:

```bash
cargo run -p aoxcmd -- economy-init --treasury-supply 1000000000000
```

Transfer from treasury to an operator account:

```bash
cargo run -p aoxcmd -- treasury-transfer --to wallet-user-01 --amount 100000
```

Delegate stake to validator:

```bash
cargo run -p aoxcmd -- stake-delegate --staker wallet-user-01 --validator validator-01 --amount 25000
```

Undelegate part of stake:

```bash
cargo run -p aoxcmd -- stake-undelegate --staker wallet-user-01 --validator validator-01 --amount 5000
```

View economy status:

```bash
cargo run -p aoxcmd -- economy-status
```

---

## 8) Production-Oriented Commands

Runtime telemetry snapshot:

```bash
cargo run -p aoxcmd -- runtime-status --trace standard --tps 25.0 --peers 8 --error-rate 0.001
```

Production audit (AI controls, genesis, staking/treasury status):

```bash
cargo run -p aoxcmd -- production-audit \
  --ai-model-signed true \
  --ai-prompt-guard true \
  --ai-anomaly-detection true \
  --ai-human-override true
```

Interop release gate:

```bash
cargo run -p aoxcmd -- interop-gate \
  --audit-complete true \
  --fuzz-complete true \
  --replay-complete true \
  --finality-matrix-complete true \
  --slo-complete true \
  --enforce
```

---

## 9) Make Targets for Daily and Release Operations

Daily checks:

```bash
make quality-quick
make quality
```

Release-focused checks and packaging:

```bash
make quality-release
make package-bin
```

Dependency security scanning:

```bash
make audit-install
make audit
```

Local supervision helper:

```bash
make supervise-local
```

---

## 10) Security Baselines and Mainnet Safeguards

- `key-bootstrap` enforces strong password requirements.
- Mainnet key generation is intentionally protected and requires explicit opt-in:
  - `--allow-mainnet`, or
  - `AOXC_ALLOW_MAINNET_KEYS=true`
- Artifact permissions are hardened on Unix-like systems for key/cert/passport outputs.
- Use enforced gates (`interop-gate --enforce`) in CI/CD, not optional reporting only.

---

## 11) Real-Network Readiness (Go/No-Go)

Do **not** claim full real-network readiness unless these are covered:

- Multi-node (3+) across distinct hosts
- Partition / restart / rejoin / recovery scenarios
- TLS/mTLS and RPC access policy hardening
- Snapshot / backup / restore runbooks tested
- Signed release artifacts + provenance verification
- Mandatory CI/CD pass/fail quality and security gates

Turkish go/no-go checklist:
- [`docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md`](docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md)

Additional references:
- [`docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md`](docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md)
- [`docs/AUDIT_READINESS_AND_OPERATIONS.md`](docs/AUDIT_READINESS_AND_OPERATIONS.md)

---

## 12) Complete `aoxcmd` Command Surface

```text
vision
compat-matrix
port-map
version
key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>]
node-bootstrap
produce-once [--tx <payload>]
node-run [--home <dir>] [--rounds <u64>] [--sleep-ms <u64>] [--tx-prefix <text>]
network-smoke [--timeout-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
real-network [--rounds <u64>] [--timeout-ms <u64>] [--pause-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
economy-status [--home <dir>] [--state <file>]
runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
interop-readiness
interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
help
```

Language support:
- `--lang <en|tr|es|de>`
- `AOXC_LANG=<code>`

---

## License
### 3.3 Continuous local node flow (`node-run`)

```bash
cargo run -p aoxcmd -- node-run --rounds 20 --sleep-ms 1000 --tx-prefix AOXC_RUN
```

What it does:
- produces multiple blocks in sequence,
- sleeps between rounds,
- returns machine-readable JSON summary (`rounds_produced`, `rounds_failed`, `final_height`).

### 3.4 Repeated network probe (`real-network`)

```bash
cargo run -p aoxcmd -- real-network --rounds 10 --timeout-ms 3000 --pause-ms 200 --bind-host 127.0.0.1 --port 0
```

What it does:
- runs repeated live TCP probe rounds,
- reports pass/fail counts,
- reports RTT min/max/avg metrics.

> Important: this is a **probe utility**, not proof of full internet-grade production P2P readiness.

---

## 4) Command reference (aoxcmd)

```text
vision
compat-matrix
port-map
version
key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>]
node-bootstrap
produce-once [--tx <payload>]
node-run [--home <dir>] [--rounds <u64>] [--sleep-ms <u64>] [--tx-prefix <text>]
network-smoke [--timeout-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
real-network [--rounds <u64>] [--timeout-ms <u64>] [--pause-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
economy-status [--home <dir>] [--state <file>]
runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
interop-readiness
interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
help
```

Language support:
- `--lang <en|tr|es|de>`
- `AOXC_LANG=<code>`

---

## 5) Security notes

- `key-bootstrap` enforces strong password baseline (length + complexity).
- `mainnet` key bootstrap is intentionally guarded and requires explicit opt-in:
  - `--allow-mainnet`, or
  - `AOXC_ALLOW_MAINNET_KEYS=true`
- Key/cert/passport outputs are written with restrictive file permissions on Unix-like systems.

---

## 6) Real-network readiness guidance

For Turkish go/no-go criteria that separate demo-level validation from operational real-chain readiness, see:

- [`docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md`](docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md)

Additional references:
- [`docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md`](docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md)
- [`docs/AUDIT_READINESS_AND_OPERATIONS.md`](docs/AUDIT_READINESS_AND_OPERATIONS.md)

---

## 7) Quality gates and CI helpers

```bash
make quality-quick
make quality
make quality-release
./scripts/quality_gate.sh quick
./scripts/quality_gate.sh full
./scripts/quality_gate.sh release
```

---

## 8) License

MIT (`LICENSE`)
