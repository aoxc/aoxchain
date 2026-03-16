<div align="center">
  <a href="https://github.com/aoxc/aoxcore">
    <img src="logos/aoxc_transparent.png" alt="AOXChain Logo" width="180" />
  </a>

# AOXChain
### Relay-First Coordination Chain on X Layer

![Status](https://img.shields.io/badge/status-live-success)
![Network](https://img.shields.io/badge/network-X%20Layer-blue)
![Model](https://img.shields.io/badge/architecture-relay--chain-purple)
![Language](https://img.shields.io/badge/stack-Rust-orange)

</div>


AOXChain is a Rust-based blockchain workspace designed as a **relay-first coordination chain**.
Its main objective is **interoperability, routing, and cross-system coordination**—not competing as a "faster L1" or "just another alternative network."

This README explains the project in a clear, chronological way: what is already live, why the chain exists, and how to run it locally.

---

## 1) Current Live Presence (X Layer References)

AOXChain already has deployed components on **X Layer**.

- **Main contract address:**
  https://www.oklink.com/tr/x-layer/address/0x97bdd1fd1caf756e00efd42eba9406821465b365/contract
- **Proxy token address:**
  https://www.oklink.com/tr/x-layer/token/0xeb9580c3946bb47d73aae1d4f7a94148b554b2f4?tab=contract
- **Multisig contract address:**
  https://www.oklink.com/tr/x-layer/address/0x20c0dd8b6559912acfac2ce061b8d5b19db8ca84/contract

These references show that AOXChain is positioned with an active, real-network footprint rather than a purely theoretical architecture.

---

## 1) Current Live Presence (X Layer References)

AOXChain already has deployed components on **X Layer**.

- **Main contract address:**
  https://www.oklink.com/tr/x-layer/address/0x97bdd1fd1caf756e00efd42eba9406821465b365/contract
- **Proxy token address:**
  https://www.oklink.com/tr/x-layer/token/0xeb9580c3946bb47d73aae1d4f7a94148b554b2f4?tab=contract
- **Multisig contract address:**
  https://www.oklink.com/tr/x-layer/address/0x20c0dd8b6559912acfac2ce061b8d5b19db8ca84/contract

These references show that AOXChain is positioned with an active, real-network footprint rather than a purely theoretical architecture.

---

## 2) Chain Purpose: Why AOXChain Exists

AOXChain is built with a **relay-chain mindset**:

- Coordinate value and messages across systems.
- Provide deterministic and auditable routing logic.
- Support governance and controlled operations through clear operator tooling.
- Prioritize reliability and interoperability over raw speed marketing.

### What AOXChain is **not**

- Not a chain focused only on maximum TPS claims.
- Not trying to be "just a different network" without a coordination role.
- Not positioned as a replacement for every execution environment.

### What AOXChain is


## 2) Chain Purpose: Why AOXChain Exists

AOXChain is built with a **relay-chain mindset**:

- Coordinate value and messages across systems.
- Provide deterministic and auditable routing logic.
- Support governance and controlled operations through clear operator tooling.
- Prioritize reliability and interoperability over raw speed marketing.

### What AOXChain is **not**

- Not a chain focused only on maximum TPS claims.
- Not trying to be "just a different network" without a coordination role.
- Not positioned as a replacement for every execution environment.

### What AOXChain is

- A practical coordination layer.
- A bridge-oriented, operations-first chain model.
- A system where security, policy, and governance controls are explicit.

---

## 3) Workspace Architecture (Rust Multi-Crate)

| Layer | Crate(s) | Responsibility |
|---|---|---|
| Protocol Core | `aoxcore` | Identity, genesis, transactions, protocol primitives |
| Consensus | `aoxcunity` | Quorum, rounds, voting, finality-oriented state |
| Networking | `aoxcnet` | Discovery, gossip, sync, transport |
| API Ingress | `aoxcrpc` | HTTP / gRPC / WebSocket surfaces |
| Execution Compatibility | `aoxcvm` | EVM/WASM/Move/UTXO-facing compatibility lanes |
| Operator Tooling | `aoxcmd`, `aoxckit` | Node lifecycle, bootstrap, economics and readiness commands |

For crate-level details, see [`crates/README.md`](crates/README.md).

---

## 4) Chronological Local Setup (Simple Path)

### Step 1 — Prerequisites


### Step 1 — Prerequisites

- Rust stable
- `cargo`
- Linux/macOS shell (recommended)

### Step 2 — Validate workspace

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

### Step 3 — Isolate runtime directory

```bash
export AOXC_HOME=$PWD/.aoxc-local
```

### Step 4 — Create key material (wallet-like identity)

```bash
cargo run -p aoxcmd -- key-bootstrap \
  --profile testnet \
  --name validator-01 \
  --password "TEST#Secure2026!"
```

### Step 5 — Initialize genesis

```bash
cargo run -p aoxcmd -- genesis-init \
  --chain-num 1001 \
  --block-time 6 \
  --treasury 1000000000000
```

### Step 6 — Bootstrap node runtime

```bash
cargo run -p aoxcmd -- node-bootstrap
```

### Step 7 — Produce first block (smoke check)

```bash
cargo run -p aoxcmd -- produce-once --tx "boot-sequence-1"
```

### Step 8 — Run node rounds

```bash
cargo run -p aoxcmd -- node-run --rounds 20 --sleep-ms 1000 --tx-prefix AOXC_RUN
```

### Step 9 — Probe network path

```bash
cargo run -p aoxcmd -- real-network \
  --rounds 10 \
  --timeout-ms 3000 \
  --pause-ms 250 \
  --bind-host 127.0.0.1 \
  --port 0
```

---

## 5) Core Operator Commands

```bash
cargo run -p aoxcmd -- version
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- port-map
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- interop-readiness
cargo run -p aoxcmd -- interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
```

---

## 6) Security and Governance Notes

- Multisig operations should remain mandatory for critical parameter changes.
- Mainnet-sensitive key generation must follow strict policy controls.
- Audit readiness should be treated as a release gate, not optional documentation.

Related docs are available under [`docs/`](docs/).

---


## 7) Enforcing AOXC Native Coin ↔ X Layer Token Equivalence in Code

To make the X Layer connection stronger at protocol level, `genesis-init` now writes a canonical settlement binding into `genesis.json`:

- `native_symbol` (default: `AOXC`)
- `native_decimals` (default: `18`)
- `settlement_network` (default: `xlayer`)
- `settlement_token_address`
- `settlement_main_contract`
- `settlement_multisig_contract`
- `equivalence_mode` (default: `1:1`)


---

## 6) Security and Governance Notes

- Multisig operations should remain mandatory for critical parameter changes.
- Mainnet-sensitive key generation must follow strict policy controls.
- Audit readiness should be treated as a release gate, not optional documentation.

Related docs are available under [`docs/`](docs/).

---


## 7) Enforcing AOXC Native Coin ↔ X Layer Token Equivalence in Code

To make the X Layer connection stronger at protocol level, `genesis-init` now writes a canonical settlement binding into `genesis.json`:

- `native_symbol` (default: `AOXC`)
- `native_decimals` (default: `18`)
- `settlement_network` (default: `xlayer`)
- `settlement_token_address`
- `settlement_main_contract`
- `settlement_multisig_contract`
- `equivalence_mode` (default: `1:1`)

Example:

```bash
cargo run -p aoxcmd -- genesis-init   --chain-num 1001   --block-time 6   --treasury 1000000000000   --native-symbol AOXC   --native-decimals 18   --settlement-network xlayer   --xlayer-token 0xeb9580c3946bb47d73aae1d4f7a94148b554b2f4   --xlayer-main-contract 0x97bdd1fd1caf756e00efd42eba9406821465b365   --xlayer-multisig 0x20c0dd8b6559912acfac2ce061b8d5b19db8ca84   --equivalence-mode 1:1
```

This settlement link is part of genesis validation and state hashing, so deployments cannot silently drift from the AOXC/X Layer contract mapping.

---
## 8) Final Positioning
## 7) Final Positioning

AOXChain should be understood as:

- **A live, X Layer-referenced system** with verifiable on-chain endpoints.
- **A relay and coordination architecture**, not a speed-only narrative.
- **An operator-first Rust workspace** designed for deterministic, auditable, and governable operation.

If your goal is secure cross-system coordination with clear governance rails, AOXChain is built for that path.
