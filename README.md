# AOXChain

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
