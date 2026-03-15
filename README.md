<div align="center">

# 🔷 AOXChain

**Interoperability-first relay chain architecture for deterministic cross-chain coordination.**

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-000000?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Workspace](https://img.shields.io/badge/Workspace-Multi%20Crate-6f42c1)](Cargo.toml)

</div>

---

## AOXChain nedir?

AOXChain, heterojen zincirler arasında **deterministik koordinasyon** hedefleyen, relay-chain odaklı bir Rust workspace'idir.

Odak noktaları:
- Zincirler arası birlikte çalışabilirlik,
- Açık ve denetlenebilir consensus/identity akışları,
- Çoklu yürütme lane modeli (EVM, WASM, Sui Move, Cardano adaptörleri),
- Operasyonel olarak testlenebilir node akışları.

## Üretim Vizyonu (Mainnet Hedefi)

1. Deterministik block üretimi + finality geçişleri,
2. Güçlü kimlik modeli (actor id, certificate, passport, PQ-ready yüzey),
3. Güvenli işletim (runbook + reproducible build + audit readiness),
4. Modüler crate sınırlarında net kontratlar.

## Repo Haritası

| Yol | Sorumluluk |
|---|---|
| `crates/aoxcore` | Çekirdek domain primitifleri (identity, tx, genesis, mempool) |
| `crates/aoxcunity` | Consensus çekirdeği (quorum, vote, proposer rotation, fork-choice, seal) |
| `crates/aoxcvm` | Çok-lane execution uyumluluk katmanı |
| `crates/aoxcnet` | Gossip/discovery/sync ağ kabuğu |
| `crates/aoxcrpc` | HTTP / gRPC / WebSocket RPC giriş katmanı |
| `crates/aoxcmd` | Node orchestration, bootstrap, deterministic smoke komutları |
| `crates/aoxckit` | Operatör araçları (keyforge vb.) |
| `crates/*` | Destekleyici crate'ler (data, ai, sdk, config, libs, exec, energy, contract...) |
| `docs/` | Mimari, audit hazırlığı, mainnet blueprint, detaylı analizler |
| `models/` | Politika/risk model örnekleri |
| `tests/` | Workspace seviyesinde entegrasyon destek yüzeyi |

## Hızlı Başlangıç

```bash
cargo check --workspace
cargo test -p aoxcmd
```

## Deterministik Operatör Akışı (`aoxcmd`)

```bash
# 1) Vizyon özeti
cargo run -p aoxcmd -- vision

# 2) Genesis üretimi
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

# 5) Tek blok deterministic üretim
cargo run -p aoxcmd -- produce-once --tx "relay-coordination-demo"

# 6) Ağ smoke
cargo run -p aoxcmd -- network-smoke

# 7) Storage smoke
cargo run -p aoxcmd -- storage-smoke --index sqlite
cargo run -p aoxcmd -- storage-smoke --index redb
```

## Operasyon ve Kalite Dokümanları

- `docs/REPO_GAP_ANALIZI_TR.md` — klasör bazlı eksik/gelişim haritası (TR).
- `docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md` — mainnet yol haritası.
- `docs/AUDIT_READINESS_AND_OPERATIONS.md` — operasyonel güvence ve audit hazırlığı.
- `docs/TEKNIK_DERIN_ANALIZ_TR.md` — teknik değerlendirme.

## Lisans

MIT (`LICENSE`).
