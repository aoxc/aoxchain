<div align="center">

# 🔷 AOXChain

**Interoperability-first relay chain architecture for deterministic cross-chain coordination.**

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-000000?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Workspace](https://img.shields.io/badge/Workspace-Multi%20Crate-6f42c1)](Cargo.toml)

</div>

---

> ⚠️ **Dürüst Durum Beyanı (Mütevazı Not)**
>
> Bu depo aktif geliştirme aşamasındadır. Mimari ve modüller üretim hedefiyle tasarlansa da,
> bağımsız üçüncü taraf güvenlik denetimi (external audit), ekonomik saldırı modellemesi,
> stres/chaos testleri ve uzun dönem operasyon verisi tamamlanmadan **mainnet için tek başına yeterli kabul edilmemelidir**.
>
> Lütfen bu projeyi körü körüne kopyalama/forklama ile doğrudan gerçek varlık yöneten ortamlara taşımayın.
> Kendi risk modeliniz, hukuki değerlendirme, güvenlik testleri ve audit süreçleriniz olmadan üretim kararı almayın.

## 1) AOXChain Nedir?

AOXChain, heterojen zincirler arasında **deterministik koordinasyon** hedefleyen, relay-chain odaklı bir Rust workspace'idir.

Odak alanları:
- zincirler arası birlikte çalışabilirlik,
- denetlenebilir consensus/identity yüzeyleri,
- çoklu yürütme lane modeli (EVM, WASM, Sui Move, Cardano adaptörleri),
- operasyonel olarak testlenebilir node akışları,
- audit-readiness ve güvenli değişiklik yönetimi.

## 2) Üretim Hedefi ve Güvenlik İlkeleri

### Mainnet hedefinin özeti
1. Deterministik block üretimi + finality geçişleri,
2. Kimlik ve sertifika tabanlı güven modeli,
3. Güçlü operasyon (runbook + reproducible build + olay müdahale),
4. Modüler crate sınırlarında net kontratlar.

### Güvenlik prensipleri
- **Default deny / explicit allow** yaklaşımı,
- **Minimum yetki** (least privilege) ve net rol ayrımı,
- **Typed error surfaces** ile izlenebilir hata yönetimi,
- **Deterministik davranış** (konsensüs kritik yüzeyde sürpriz yok),
- **Audit izi**: dokümantasyon + test + PR disiplininin birlikte sürdürülmesi.

## 3) Hızlı Başlangıç

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 4) Repo Haritası

| Yol | Sorumluluk |
|---|---|
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

# 8) Ekonomi bootstrap (hazine + stake)
cargo run -p aoxcmd -- economy-init --treasury-supply 1000000000000
cargo run -p aoxcmd -- treasury-transfer --to validator-1 --amount 500000000
cargo run -p aoxcmd -- stake-delegate --staker validator-1 --validator val-core-1 --amount 250000000
cargo run -p aoxcmd -- economy-status
```

## 6) Dev/Testnet Kurulum Referansları

- Local script: [`scripts/run-local.sh`](scripts/run-local.sh)
- Konfigürasyonlar: [`configs/mainnet.toml`](configs/mainnet.toml), [`configs/testnet.toml`](configs/testnet.toml), [`configs/genesis.json`](configs/genesis.json)
- Container seti: [`Dockerfile`](Dockerfile), [`docker-compose.yaml`](docker-compose.yaml)

> Not: Bu repo şu anda “kolay kurulum” yönünde ilerlemektedir; üretim-grade otomatik orkestrasyon,
> uzun süreli fault-injection testleri ve tam runbook standardizasyonu sürekli geliştirme konusudur.

## 7) SDK ve Entegrasyon Başlangıcı

AOXChain SDK yüzeyi için başlangıç noktası:
- **[`crates/aoxcsdk/README.md`](crates/aoxcsdk/README.md)**

SDK, istemci tarafı entegrasyonlarında stabilize API hedefiyle geliştirilir; sürüm geçişlerinde değişiklik notlarını takip edin.

## 8) Dokümantasyon Merkezi

### Operasyon + Audit
- [`docs/AUDIT_READINESS_AND_OPERATIONS.md`](docs/AUDIT_READINESS_AND_OPERATIONS.md)
- [`docs/P2P_AUDIT_GUIDE_EN.md`](docs/P2P_AUDIT_GUIDE_EN.md)

### Mimari + Yol Haritası
- [`docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md`](docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md)
- [`docs/TEKNIK_DERIN_ANALIZ_TR.md`](docs/TEKNIK_DERIN_ANALIZ_TR.md)
- [`docs/REPO_GAP_ANALIZI_TR.md`](docs/REPO_GAP_ANALIZI_TR.md)

### Sorumlu kullanım ve risk bildirimi
- [`docs/SECURITY_AND_RISK_NOTICE_TR.md`](docs/SECURITY_AND_RISK_NOTICE_TR.md)

## 9) Katkı ve Güvenlik Disiplini

- Konsensüs/kimlik/ağ yüzeyine dokunan değişikliklerde test zorunludur.
- Lint temizliği (`clippy -D warnings`) korunmalıdır.
- Büyük değişikliklerde tasarım notu + tehdit modeli + rollback planı önerilir.
- Operasyonel güvenlik için anahtar materyal, sertifika ve gizli dosyalar ayrı yönetilmelidir.

## 10) Lisans

MIT (`LICENSE`).
