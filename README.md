# AOXChain

AOXChain is being designed as an **interoperable relay-oriented coordination chain**, not as a pure throughput-first monolith. The strategic objective is to provide deterministic coordination, cross-chain compatibility, and robust identity/consensus primitives that can interoperate with heterogeneous execution ecosystems.

> Status: pre-mainnet engineering. Workspace compiles, CLI smoke path is operational, and mainnet-hardening tracks are documented.

## Strategic Vision

AOXChain is intended to:

1. Operate as a **relay and coordination layer** across multiple chains.
2. Prioritize **determinism, compatibility, and auditability** over short-term TPS optimization.
3. Support **future-proof identity and trust surfaces** (post-quantum capable key/cert/passport pipeline).
4. Provide a **multi-lane architecture** for heterogeneous contract/runtime ecosystems.

## Workspace Topology

- `aoxcore`: identity, genesis, transaction hashing, mempool.
- `aoxcunity`: consensus, quorum, proposer rotation, fork-choice, finalization surfaces.
- `aoxcvm`: lane execution compatibility abstractions.
- `aoxcnet`: networking/gossip/discovery shell.
- `aoxcmd`: operational CLI for bootstrap and deterministic smoke execution.
- `aoxcrpc`, `aoxcsdk`, `aoxckit`, and others: integration/tooling layers.

## Build and Core Validation

```bash
cargo check --workspace
cargo test -p aoxcmd
cargo test -p aoxcunity
```

## End-to-End CLI Bootstrap (Current Deterministic Path)

### 1) Chain vision introspection

```bash
cargo run -p aoxcmd -- vision
```

### 2) Genesis creation

```bash
cargo run -p aoxcmd -- genesis-init \
  --path AOXC_DATA/identity/genesis.json \
  --chain-num 1 \
  --block-time 6 \
  --treasury 1000000000
```

### 3) Key + identity material bootstrap
AOXChain is a modular Rust workspace that explores a multi-lane blockchain architecture with explicit separation between core state, consensus, networking, runtime orchestration, RPC, and AI-assisted policy surfaces.

> **Current stage:** pre-mainnet engineering. The repository now compiles at workspace level and exposes a deterministic CLI smoke path for key bootstrap + node bootstrap + single-block production.

## Architecture Overview

- `crates/aoxcore` — genesis, identity, transaction, and mempool domain primitives.
- `crates/aoxcunity` — consensus state machine, proposer rotation, quorum logic, vote pool, fork-choice.
- `crates/aoxcvm` — lane-oriented execution abstractions (EVM/WASM/Sui/Cardano compatibility surface).
- `crates/aoxcnet` — p2p/discovery/gossip synchronization shell (transport integration still staged).
- `crates/aoxcrpc` — HTTP/gRPC/WebSocket API surfaces.
- `crates/aoxcmd` — operational node bootstrap and orchestration CLI.
- `crates/aoxckit` — keyforge and operational helper toolkit.

## Build and Validation

```bash
cargo check --workspace
cargo test -p aoxcunity -- --nocapture
```

## AOXCMD Operational CLI (Deterministic Smoke Flow)

### 1) Bootstrap key material

```bash
cargo run -p aoxcmd -- key-bootstrap \
  --password "change-me" \
  --base-dir AOXC_DATA/keys \
  --name relay-1 \
  --chain AOXC-MAIN \
  --role relay \
  --zone global \
  --name validator-1 \
  --chain AOXC-MAIN \
  --role validator \
  --zone core \
  --issuer AOXC-ROOT-CA \
  --validity-secs 31536000
```

### 4) Node bootstrap validation
### 2) Bootstrap node state

```bash
cargo run -p aoxcmd -- node-bootstrap
```

### 5) Produce and finalize one block (deterministic smoke)

```bash
cargo run -p aoxcmd -- produce-once --tx "relay-coordination-demo"
```

### 6) Network integration stub check
=======
### 3) Produce and finalize one block (smoke path)

```bash
cargo run -p aoxcmd -- produce-once --tx "hello-mainnet-path"
```

### 4) Verify network stub wiring

```bash
cargo run -p aoxcmd -- network-smoke
```

## Mainnet Hardening Priorities

- Transport-backed gossip and peer routing in `aoxcnet`.
- Multi-node integration tests (`proposal -> vote -> finalize` lifecycle).
- RPC/runtime persistent-state integration.
- Threat modeling, adversarial simulation, and external audit report.
- Release attestation and reproducible build pipeline.

## Engineering Documents

- `docs/AUDIT_READINESS_AND_OPERATIONS.md`
- `docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md`

## License
=======
## Security and Audit Readiness Principles

The codebase is being shaped around audit-friendly engineering constraints:

1. **Explicit error surfaces** (typed errors and panic minimization in critical flows).
2. **Deterministic protocol operations** (canonical builders and stable hashing domains).
3. **Resource-bounded subsystems** (mempool size, TTL, and payload constraints).
4. **Atomic persistence patterns** (temporary file + fsync + rename where applicable).
5. **Separation of cryptographic authority from runtime orchestration** (CA and key lifecycle abstractions).

A dedicated engineering document is provided at:

- `docs/AUDIT_READINESS_AND_OPERATIONS.md`

## Mainnet Readiness Gate (High Level)

- [x] Workspace-level compile integrity (`cargo check --workspace`)
- [x] Consensus unit test execution (`aoxcunity`)
- [x] Deterministic one-block production smoke flow via CLI
- [ ] End-to-end multi-node transport-backed gossip
- [ ] Full RPC-to-runtime integration with persistent state backends
- [ ] Formal threat model + external security review + reproducible release pipeline

## License
=======
AOXChain, modüler Rust workspace mimarisi ile geliştirilen çok-katmanlı bir zincir çekirdeği denemesidir.

> Durum: aktif geliştirme / pre-mainnet.

## Mimari harita

Bu repo birden fazla crate içerir ve katmanlar bilinçli olarak ayrıştırılmıştır:

- `crates/aoxcore`: genesis, identity, transaction, mempool ve temel domain modelleri.
- `crates/aoxcunity`: consensus, fork-choice, validator rotation, vote/quorum yapıları.
- `crates/aoxcvm`: lane tabanlı execution uyumluluğu (EVM/WASM/Sui/Cardano).
- `crates/aoxcnet`: p2p/gossip/discovery/sync iskeleti.
- `crates/aoxcrpc`: HTTP, gRPC, WebSocket API katmanı.
- `crates/aoxcmd`: node bootstrap/orchestration (runtime wiring).
- `crates/aoxcai`: AI policy/engine/backends prototipleri.
- `crates/aoxckit`: operational CLI/tooling bileşenleri.

## Hızlı başlangıç

### Gereksinimler

- Rust stable toolchain (2024 edition destekli)
- `cargo`

### Derleme ve test

```bash
cargo check --workspace
cargo test --workspace
```

### Seçili crate kontrolleri

```bash
cargo check -p aoxcore
cargo check -p aoxcunity
cargo check -p aoxcvm
cargo check -p aoxcmd
cargo check -p aoxcnet
```

## Geliştirme yaklaşımı

1. Önce katman kontratları (`aoxcmd <-> aoxcunity`, `aoxcmd <-> aoxcore`) sabitlenir.
2. Sonra network ve rpc yüzeyleri gerçek transport ile bağlanır.
3. En sonda performans/güvenlik sertleştirmesi ve mainnet parametreleri yapılır.

## Mainnet'e yakınlık için checklist

- [ ] Workspace sürekli derlenebilir (`cargo check --workspace` yeşil)
- [ ] Workspace testleri stabil (`cargo test --workspace` yeşil)
- [ ] Node bootstrap + proposal + vote + finalize integration test
- [ ] Gossip transport gerçek p2p peer graph ile bağlı
- [ ] RPC katmanı canlı node state ile uçtan uca entegre
- [ ] Deterministic replay / state transition testleri
- [ ] Operasyon dokümantasyonu (runbook, key yönetimi, incident akışı)

## Dizin notları

- `models/`: örnek model/policy yaml dosyaları.
- `tests/`: workspace-level test crate.
- `crates/aoxcdata/logs/`: local geliştirme log örnekleri.

## Katkı prensipleri

- Önce compile düzeyi bozulmadan küçük PR’lar.
- API değişiminde kullanan crate’leri aynı PR’da güncelleme.
- Test ve dokümantasyon güncellemesi olmadan kritik katman değişikliği yapmama.

## Lisans

MIT (`LICENSE`).
