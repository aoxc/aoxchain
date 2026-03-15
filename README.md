<div align="center">

# 🔷 AOXChain

**Interoperability-first relay chain architecture for deterministic cross-chain coordination.**

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-000000?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Workspace](https://img.shields.io/badge/Workspace-Multi%20Crate-6f42c1)](Cargo.toml)

</div>

---

> ⚠️ **Project Status **
>
> This repository is under active development. While the architecture and modules target production use,
> it should **not** be treated as mainnet-ready without independent third-party security audits,
> economic attack simulations, stress/chaos testing, and long-term operational evidence.
>
> Please do not blindly copy/fork this project into environments that manage real assets.
> Do not make production decisions without your own risk model, legal review, security testing,
> and audit process.

## 1) What is AOXChain?

AOXChain is a relay-chain-oriented Rust workspace designed for **deterministic cross-chain coordination**.

Core focus areas:
- interoperability across heterogeneous chains,
- auditable consensus and identity surfaces,
- multi-lane execution model (EVM, WASM, Sui Move, Cardano adapters),
- operationally testable node workflows,
- audit-readiness and disciplined change management.

## 2) Production Goals and Security Principles

### Mainnet target summary
1. Deterministic block production and finality transitions,
2. Identity and certificate-based trust model,
3. Strong operations (runbooks, reproducible builds, incident response),
4. Clear crate boundaries and explicit contracts.

### Security principles
- **Default deny / explicit allow**,
- **Least privilege** and clear role separation,
- **Typed error surfaces** for traceable failures,
- **Deterministic behavior** on consensus-critical paths,
- **Audit trail discipline** across documentation, tests, and pull requests.

## 3) Quick Start

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```


## 5) Deterministic Operator Flow (`aoxcmd`)
| `crates/aoxcore` | Çekirdek domain primitifleri (identity, tx, genesis, mempool) |
| `crates/aoxcunity` | Consensus çekirdeği (quorum, vote, proposer rotation, fork-choice, seal) |
| `crates/aoxcvm` | Çok-lane execution uyumluluk katmanı |
| `crates/aoxcnet` | Gossip/discovery/sync ağ kabuğu |
| `crates/aoxcrpc` | HTTP / gRPC / WebSocket RPC giriş katmanı |
| `crates/aoxcmd` | Node orchestration ve deterministic operatör komutları |
| `crates/aoxckit` | Keyforge ve operasyonel kriptografik araçlar |
| `crates/aoxcsdk` | Uygulama/entegrasyon geliştiricileri için SDK yüzeyi |
| `docs/` | Mimari, audit hazırlığı, runbook, risk ve analiz dokümantasyonu |

Detaylı crate dizini: **[`crates/README.md`](crates/README.md)**

## 5) Deterministik Operatör Akışı (`aoxcmd`)

```bash
# 1) Vision summary
cargo run -p aoxcmd -- vision

# 2) Generate genesis
cargo run -p aoxcmd -- genesis-init \
  --path AOXC_DATA/identity/genesis.json \
  --chain-num 1 \
  --block-time 6 \
  --treasury 1000000000

# 3) Key + identity bootstrap
cargo run -p aoxcmd -- key-bootstrap \
  --password "change-me" \
  --base-dir AOXC_DATA/keys \
  --name validator-1 \
  --chain AOXC-MAIN \
  --role validator \
  --zone core \
  --issuer AOXC-ROOT-CA \
  --validity-secs 31536000

# 4) Node bootstrap
cargo run -p aoxcmd -- node-bootstrap

# 5) Produce a deterministic single block
cargo run -p aoxcmd -- produce-once --tx "relay-coordination-demo"

# 6) Network smoke
cargo run -p aoxcmd -- network-smoke

# 7) Storage smoke
cargo run -p aoxcmd -- storage-smoke --index sqlite
cargo run -p aoxcmd -- storage-smoke --index redb

# 8) Economy bootstrap (treasury + staking)

# 8) Ekonomi bootstrap (hazine + stake)
cargo run -p aoxcmd -- economy-init --treasury-supply 1000000000000
cargo run -p aoxcmd -- treasury-transfer --to validator-1 --amount 500000000
cargo run -p aoxcmd -- stake-delegate --staker validator-1 --validator val-core-1 --amount 250000000
cargo run -p aoxcmd -- economy-status
```

## 6) Dev/Testnet Setup References

- Local script: [`scripts/run-local.sh`](scripts/run-local.sh)
- Config profiles: [`configs/mainnet.toml`](configs/mainnet.toml), [`configs/testnet.toml`](configs/testnet.toml), [`configs/genesis.json`](configs/genesis.json)
- Container setup: [`Dockerfile`](Dockerfile), [`docker-compose.yaml`](docker-compose.yaml)

> Note: The repository is actively evolving toward easier setup. Production-grade automated orchestration,
> long-running fault injection coverage, and full runbook standardization are still ongoing work.

## 7) SDK and Integration Entry Point

Start here for the AOXChain SDK surface:
- **[`crates/aoxcsdk/README.md`](crates/aoxcsdk/README.md)**

The SDK evolves toward stable integration APIs; track release notes for compatibility changes.

## 8) Documentation Hub

### Operations + Audit
- [`docs/AUDIT_READINESS_AND_OPERATIONS.md`](docs/AUDIT_READINESS_AND_OPERATIONS.md)
- [`docs/P2P_AUDIT_GUIDE_EN.md`](docs/P2P_AUDIT_GUIDE_EN.md)

### Architecture + Roadmap
- [`docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md`](docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md)
- [`docs/TEKNIK_DERIN_ANALIZ_TR.md`](docs/TEKNIK_DERIN_ANALIZ_TR.md)
- [`docs/REPO_GAP_ANALIZI_TR.md`](docs/REPO_GAP_ANALIZI_TR.md)

### Responsible use and risk notice
- [`docs/SECURITY_AND_RISK_NOTICE_TR.md`](docs/SECURITY_AND_RISK_NOTICE_TR.md)

## 9) Contribution and Security Discipline

- Changes touching consensus/identity/networking must include tests.
- Keep linting clean (`clippy -D warnings`).
- For large changes, include a design note, threat model update, and rollback plan.
- Keep key material, certificates, and sensitive artifacts under strict operational controls.

## 10) License
## 6) Dev/Testnet Kurulum Referansları

- Local script: [`scripts/run-local.sh`](scripts/run-local.sh)
- Konfigürasyonlar: [`configs/mainnet.toml`](configs/mainnet.toml), [`configs/testnet.toml`](configs/testnet.toml), [`configs/genesis.json`](configs/genesis.json)
- Container seti: [`Dockerfile`](Dockerfile), [`docker-compose.yaml`](docker-compose.yaml)


## 10) Lisans

MIT (`LICENSE`).
