<div align="center">
  <a href="https://github.com/aoxc/aoxcore">
    <img src="logos/aoxc_transparent.png" alt="AOXChain Logo" width="180" />
  </a>

# AOXChain
### Experimental Sovereign Coordination Chain
#### AOXC Alpha: Genesis V1

![Status](https://img.shields.io/badge/status-experimental-orange)
![Model](https://img.shields.io/badge/architecture-sovereign--core-purple)
![Stack](https://img.shields.io/badge/stack-rust-orange)
![CLI](https://img.shields.io/badge/tooling-aoxcmd-blue)

</div>

AOXChain is an experimental Rust blockchain workspace built around a simple idea:

> **the local chain is the sovereign constitutional core, and remote systems are execution domains.**

This repository should be read as a **new chain project**. It is not positioned as a wrapper around another network, and this README intentionally describes AOXChain on its own terms.

---

## 1. What AOXChain is

AOXChain is designed to own the parts of a system that must remain canonical:

- identity,
- supply,
- governance,
- relay authorization,
- validator/security policy,
- settlement finality,
- treasury and reserves.

Remote domains may execute logic, hold contract adapters, and provide ecosystem-specific integrations, but the **final constitutional authority** stays on AOXChain.

---

## 2. Current architecture in one sentence

- **Local chain:** sovereign constitutional core.
- **Remote chains/domains:** execution, integration, liquidity, and application surfaces.

If you want the machine-readable view, run:

```bash
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- sovereign-core
cargo run -p aoxcmd -- module-architecture
```

---

## 3. Canonical local roots

AOXChain currently models the following local constitutional roots:

1. `identity`
2. `supply`
3. `governance`
4. `relay`
5. `security`
6. `settlement`
7. `treasury`

These can be inspected from the CLI:

```bash
cargo run -p aoxcmd -- sovereign-core
```

---

## 4. Address and key derivation format

AOXChain uses a BIP44-style derivation prefix centered on the AOXC coin type.

### Canonical HD path

```text
m/44/2626/<chain>/<role>/<zone>/<index>
```

Example:

```text
m/44/2626/1/1/2/0
```

Meaning:

- `44` -> BIP44 purpose
- `2626` -> AOXC coin type / chain identity namespace
- `chain` -> chain identifier
- `role` -> actor role
- `zone` -> logical or geographic zone
- `index` -> sequential key index

This path model is implemented in the AOXC identity layer and should be treated as the canonical derivation format for operator and system key material.

---

## 5. Workspace layout

| Layer | Crate(s) | Responsibility |
|---|---|---|
| Protocol | `aoxcore` | identity, protocol primitives, genesis, tx, receipts |
| Consensus | `aoxcunity` | rounds, quorum, vote/finality state |
| Networking | `aoxcnet` | transport, discovery, gossip, sync |
| RPC / Ingress | `aoxcrpc` | HTTP, gRPC, WebSocket, security middleware |
| Execution | `aoxcvm` | multi-lane runtime and compatibility layers |
| Operations | `aoxcmd`, `aoxckit` | bootstrap, runtime ops, manifests, policy commands |

---

## 6. Fast local start


## 2. Current architecture in one sentence

- **Local chain:** sovereign constitutional core.
- **Remote chains/domains:** execution, integration, liquidity, and application surfaces.

If you want the machine-readable view, run:

```bash
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- sovereign-core
cargo run -p aoxcmd -- module-architecture
```

---

## 3. Canonical local roots

AOXChain currently models the following local constitutional roots:

1. `identity`
2. `supply`
3. `governance`
4. `relay`
5. `security`
6. `settlement`
7. `treasury`

These can be inspected from the CLI:

```bash
cargo run -p aoxcmd -- sovereign-core
```

---

## 4. Address and key derivation format

AOXChain uses a BIP44-style derivation prefix centered on the AOXC coin type.

### Canonical HD path

```text
m/44/2626/<chain>/<role>/<zone>/<index>
```

Example:

```text
m/44/2626/1/1/2/0
```

Meaning:

- `44` -> BIP44 purpose
- `2626` -> AOXC coin type / chain identity namespace
- `chain` -> chain identifier
- `role` -> actor role
- `zone` -> logical or geographic zone
- `index` -> sequential key index

This path model is implemented in the AOXC identity layer and should be treated as the canonical derivation format for operator and system key material.

---

## 5. Workspace layout

| Layer | Crate(s) | Responsibility |
|---|---|---|
| Protocol | `aoxcore` | identity, protocol primitives, genesis, tx, receipts |
| Consensus | `aoxcunity` | rounds, quorum, vote/finality state |
| Networking | `aoxcnet` | transport, discovery, gossip, sync |
| RPC / Ingress | `aoxcrpc` | HTTP, gRPC, WebSocket, security middleware |
| Execution | `aoxcvm` | multi-lane runtime and compatibility layers |
| Operations | `aoxcmd`, `aoxckit` | bootstrap, runtime ops, manifests, policy commands |

---


## 2. Current architecture in one sentence

- **Local chain:** sovereign constitutional core.
- **Remote chains/domains:** execution, integration, liquidity, and application surfaces.

If you want the machine-readable view, run:

```bash
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- sovereign-core
cargo run -p aoxcmd -- module-architecture
```

---

## 3. Canonical local roots

AOXChain currently models the following local constitutional roots:

1. `identity`
2. `supply`
3. `governance`
4. `relay`
5. `security`
6. `settlement`
7. `treasury`

These can be inspected from the CLI:

```bash
cargo run -p aoxcmd -- sovereign-core
```

---

## 4. Address and key derivation format

AOXChain uses a BIP44-style derivation prefix centered on the AOXC coin type.

### Canonical HD path

```text
m/44/2626/<chain>/<role>/<zone>/<index>
```

Example:

```text
m/44/2626/1/1/2/0
```

Meaning:

- `44` -> BIP44 purpose
- `2626` -> AOXC coin type / chain identity namespace
- `chain` -> chain identifier
- `role` -> actor role
- `zone` -> logical or geographic zone
- `index` -> sequential key index


Meaning:

- `44` -> BIP44 purpose
- `2626` -> AOXC coin type / chain identity namespace
- `chain` -> chain identifier
- `role` -> actor role
- `zone` -> logical or geographic zone
- `index` -> sequential key index

This path model is implemented in the AOXC identity layer and should be treated as the canonical derivation format for operator and system key material.

---

## 5. Workspace layout

| Layer | Crate(s) | Responsibility |
|---|---|---|
| Protocol | `aoxcore` | identity, protocol primitives, genesis, tx, receipts |
| Consensus | `aoxcunity` | rounds, quorum, vote/finality state |
| Networking | `aoxcnet` | transport, discovery, gossip, sync |
| RPC / Ingress | `aoxcrpc` | HTTP, gRPC, WebSocket, security middleware |
| Execution | `aoxcvm` | multi-lane runtime and compatibility layers |
| Operations | `aoxcmd`, `aoxckit` | bootstrap, runtime ops, manifests, policy commands |

---

## 6. Fast local start


## 6. Fast local start

### Prerequisites

- Rust stable
- `cargo`
- Linux/macOS shell

### Validate workspace

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

### Create isolated runtime directory

```bash
export AOXC_HOME="$PWD/.aoxc-local"
umask 077
mkdir -p "$AOXC_HOME"
```

### Bootstrap operator keys

```bash
cargo run -p aoxcmd -- key-bootstrap \
  --home "$AOXC_HOME" \
  --profile testnet \
  --name validator-01 \
  --password 'TEST#Secure2026!'
```

### Initialize genesis

```bash
cargo run -p aoxcmd -- genesis-init \
  --home "$AOXC_HOME" \
  --chain-num 1001 \
  --block-time 6 \
  --treasury 1000000000000
```

### Bootstrap the node

```bash
cargo run -p aoxcmd -- node-bootstrap --home "$AOXC_HOME"
```

### Produce a smoke block

```bash
cargo run -p aoxcmd -- produce-once --home "$AOXC_HOME" --tx 'boot-sequence-1'
```

### Run bounded rounds

```bash
cargo run -p aoxcmd -- node-run \
  --home "$AOXC_HOME" \
  --rounds 20 \
  --sleep-ms 1000 \
  --tx-prefix AOXC_RUN
```

---

## 7. Developer workflow with `make`

AOXChain includes a `Makefile` for the common local workflow.

### Discover available targets

```bash
make help
```

### Most useful targets

```bash
make fmt
make check
make test
make clippy
make quality-quick
make quality
make build-release
make package-bin
make version
make manifest
make policy

```bash
make fmt
make check
make test
make clippy
make quality-quick
make quality
make build-release
make package-bin
make version
make manifest
make policy

```bash
make fmt
make check
make test
make clippy
make quality-quick
make quality
make build-release
make package-bin
make version
make manifest
make policy
```

### Real local chain loop

```bash
make real-chain-run-once
make real-chain-run
make real-chain-tail
make real-chain-health
```

### Release-oriented flow

```bash
make quality-release
make package-bin
make manifest
make policy

## 7. Developer workflow with `make`

AOXChain includes a `Makefile` for the common local workflow.

### Discover available targets

```bash
make help
```

### Most useful targets

```bash
make fmt
make check
make test
make clippy
make quality-quick
make quality
make build-release
make package-bin
make version
make manifest
make policy
```

### Real local chain loop

```bash
make real-chain-run-once
make real-chain-run
make real-chain-tail
make real-chain-health
```

### Release-oriented flow

## 8. Important CLI commands

### Chain identity and architecture

```bash
cargo run -p aoxcmd -- version
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- sovereign-core
cargo run -p aoxcmd -- module-architecture
cargo run -p aoxcmd -- compat-matrix
```

### Build and supply-chain visibility

```bash
cargo run -p aoxcmd -- build-manifest
cargo run -p aoxcmd -- node-connection-policy
cargo run -p aoxcmd -- node-connection-policy --enforce-official
```

### Runtime and network inspection

```bash
cargo run -p aoxcmd -- port-map
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- network-smoke --timeout-ms 3000 --bind-host 127.0.0.1 --port 0
cargo run -p aoxcmd -- real-network --rounds 10 --timeout-ms 3000 --pause-ms 250 --bind-host 127.0.0.1 --port 0
```

### Real local chain loop

```bash
make real-chain-run-once
make real-chain-run
make real-chain-tail
make real-chain-health
```

### Release-oriented flow

```bash
make quality-release
make package-bin
make manifest
make policy
```
## 9. Security posture
```bash
make quality-release
make package-bin
make manifest
make policy
```

---

## 8. Important CLI commands

### Chain identity and architecture

```bash
cargo run -p aoxcmd -- version
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- sovereign-core
cargo run -p aoxcmd -- module-architecture
cargo run -p aoxcmd -- compat-matrix
```

### Build and supply-chain visibility

```bash
cargo run -p aoxcmd -- build-manifest
cargo run -p aoxcmd -- node-connection-policy
cargo run -p aoxcmd -- node-connection-policy --enforce-official
```

### Runtime and network inspection

```bash
cargo run -p aoxcmd -- port-map
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- network-smoke --timeout-ms 3000 --bind-host 127.0.0.1 --port 0
cargo run -p aoxcmd -- real-network --rounds 10 --timeout-ms 3000 --pause-ms 250 --bind-host 127.0.0.1 --port 0
```

---

## 9. Security posture
## 8. Important CLI commands

### Chain identity and architecture

```bash
cargo run -p aoxcmd -- version
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- sovereign-core
cargo run -p aoxcmd -- module-architecture
cargo run -p aoxcmd -- compat-matrix
```

### Build and supply-chain visibility

```bash
cargo run -p aoxcmd -- build-manifest
cargo run -p aoxcmd -- node-connection-policy
cargo run -p aoxcmd -- node-connection-policy --enforce-official
```

### Runtime and network inspection

```bash
cargo run -p aoxcmd -- port-map
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- network-smoke --timeout-ms 3000 --bind-host 127.0.0.1 --port 0
cargo run -p aoxcmd -- real-network --rounds 10 --timeout-ms 3000 --pause-ms 250 --bind-host 127.0.0.1 --port 0
```

AOXChain is still **experimental**.

That means:

- do not market it as finished mainnet software,
- do not assume all remote-domain threat models are closed,
- do not treat local ad-hoc builds as production artifacts,
- do not peer production nodes without certificate and release policy validation.

Recommended production direction:

- embedded node certificate,
- attestation hash exchange,
- mTLS,
- signed release manifest,
- external audit,
- replay/finality test matrix,
- multi-node adversarial simulation.

---

## 10. Docs worth reading next

- `READ.md` -> audit-style operator flow
- `docs/SOVEREIGN_CORE_MODEL_TR.md` -> local constitutional core model
- `docs/FIVE_MODULE_RELAY_ARCHITECTURE_TR.md` -> module layout
- `crates/README.md` -> crate map

---

## 11. Current truth about readiness

AOXChain is no longer only a concept repo: it has protocol modeling, CLI tooling, build metadata, and deterministic local flows.

But it is still **not complete**.

If you want to raise confidence toward `~75%` engineering readiness, the next highest-value additions would be:

1. more deterministic integration tests,
2. remote-domain contract skeletons,
3. attestation-aware peer handshake,
4. cert issue/rotate/revoke CLI,
5. release manifest signing,
6. structured terminal dashboard and richer operator logs,
7. fuzzing and replay suites.

That is the path from experimental chain to production candidate.

## Deterministic Testnet Fixture

A deterministic 5-node **test-only** network fixture now exists under `configs/deterministic-testnet/`, with public seeds, funded genesis accounts, node TOML files, and a runnable `launch-testnet.sh` helper. See `docs/DETERMINISTIC_TESTNET_TR.md` for the Turkish operator guide.
A local benchmark and quantified mainnet-readiness report are also available via `aoxcmd` (`load-benchmark` and `mainnet-readiness`). See `docs/LOCAL_BENCHMARK_AND_MAINNET_READINESS_TR.md`.
For the next phase, distributed validation and operator workflow are documented in `docs/REAL_NETWORK_VALIDATION_RUNBOOK_TR.md`, and the evidence-backed readiness checklist lives in `models/mainnet_readiness_evidence_v1.yaml`.
## 9. Security posture
```bash
cargo run -p aoxcmd -- version
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- sovereign-core
cargo run -p aoxcmd -- module-architecture
cargo run -p aoxcmd -- compat-matrix
```

### Build and supply-chain visibility

```bash
cargo run -p aoxcmd -- build-manifest
cargo run -p aoxcmd -- node-connection-policy
cargo run -p aoxcmd -- node-connection-policy --enforce-official
```

### Runtime and network inspection

```bash
cargo run -p aoxcmd -- port-map
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- network-smoke --timeout-ms 3000 --bind-host 127.0.0.1 --port 0
cargo run -p aoxcmd -- real-network --rounds 10 --timeout-ms 3000 --pause-ms 250 --bind-host 127.0.0.1 --port 0
```

AOXChain is still **experimental**.

That means:

- do not market it as finished mainnet software,
- do not assume all remote-domain threat models are closed,
- do not treat local ad-hoc builds as production artifacts,
- do not peer production nodes without certificate and release policy validation.

Recommended production direction:

- embedded node certificate,
- attestation hash exchange,
- mTLS,
- signed release manifest,
- external audit,
- replay/finality test matrix,
- multi-node adversarial simulation.

AOXChain is still **experimental**.

That means:

- do not market it as finished mainnet software,
- do not assume all remote-domain threat models are closed,
- do not treat local ad-hoc builds as production artifacts,
- do not peer production nodes without certificate and release policy validation.

Recommended production direction:

- embedded node certificate,
- attestation hash exchange,
- mTLS,
- signed release manifest,
- external audit,
- replay/finality test matrix,
- multi-node adversarial simulation.

---

## 10. Docs worth reading next

- `READ.md` -> audit-style operator flow
- `docs/SOVEREIGN_CORE_MODEL_TR.md` -> local constitutional core model
- `docs/FIVE_MODULE_RELAY_ARCHITECTURE_TR.md` -> module layout
- `crates/README.md` -> crate map

---

## 11. Current truth about readiness

AOXChain is no longer only a concept repo: it has protocol modeling, CLI tooling, build metadata, and deterministic local flows.

But it is still **not complete**.

If you want to raise confidence toward `~75%` engineering readiness, the next highest-value additions would be:

1. more deterministic integration tests,
2. remote-domain contract skeletons,
3. attestation-aware peer handshake,
4. cert issue/rotate/revoke CLI,
5. release manifest signing,
6. structured terminal dashboard and richer operator logs,
7. fuzzing and replay suites.

That is the path from experimental chain to production candidate.

## Deterministic Testnet Fixture

A deterministic 5-node **test-only** network fixture now exists under `configs/deterministic-testnet/`, with public seeds, funded genesis accounts, node TOML files, and a runnable `launch-testnet.sh` helper. See `docs/DETERMINISTIC_TESTNET_TR.md` for the Turkish operator guide.
A local benchmark and quantified mainnet-readiness report are also available via `aoxcmd` (`load-benchmark` and `mainnet-readiness`). See `docs/LOCAL_BENCHMARK_AND_MAINNET_READINESS_TR.md`.
## 10. Docs worth reading next

- `READ.md` -> audit-style operator flow
- `docs/SOVEREIGN_CORE_MODEL_TR.md` -> local constitutional core model
- `docs/FIVE_MODULE_RELAY_ARCHITECTURE_TR.md` -> module layout
- `crates/README.md` -> crate map

---

## 11. Current truth about readiness

AOXChain is no longer only a concept repo: it has protocol modeling, CLI tooling, build metadata, and deterministic local flows.

But it is still **not complete**.

If you want to raise confidence toward `~75%` engineering readiness, the next highest-value additions would be:

1. more deterministic integration tests,
2. remote-domain contract skeletons,
3. attestation-aware peer handshake,
4. cert issue/rotate/revoke CLI,
5. release manifest signing,
6. structured terminal dashboard and richer operator logs,
7. fuzzing and replay suites.

That is the path from experimental chain to production candidate.

## Deterministic Testnet Fixture

A deterministic 5-node **test-only** network fixture now exists under `configs/deterministic-testnet/`, with public seeds, funded genesis accounts, node TOML files, and a runnable `launch-testnet.sh` helper. See `docs/DETERMINISTIC_TESTNET_TR.md` for the Turkish operator guide.
## 9. Security posture

AOXChain is still **experimental**.

That means:

- do not market it as finished mainnet software,
- do not assume all remote-domain threat models are closed,
- do not treat local ad-hoc builds as production artifacts,
- do not peer production nodes without certificate and release policy validation.

Recommended production direction:

- embedded node certificate,
- attestation hash exchange,
- mTLS,
- signed release manifest,
- external audit,
- replay/finality test matrix,
- multi-node adversarial simulation.

---

## 10. Docs worth reading next

- `READ.md` -> audit-style operator flow
- `docs/SOVEREIGN_CORE_MODEL_TR.md` -> local constitutional core model
- `docs/FIVE_MODULE_RELAY_ARCHITECTURE_TR.md` -> module layout
- `crates/README.md` -> crate map

---

## 11. Current truth about readiness

AOXChain is no longer only a concept repo: it has protocol modeling, CLI tooling, build metadata, and deterministic local flows.

But it is still **not complete**.

If you want to raise confidence toward `~75%` engineering readiness, the next highest-value additions would be:

1. more deterministic integration tests,
2. remote-domain contract skeletons,
3. attestation-aware peer handshake,
4. cert issue/rotate/revoke CLI,
5. release manifest signing,
6. structured terminal dashboard and richer operator logs,
7. fuzzing and replay suites.

That is the path from experimental chain to production candidate.
