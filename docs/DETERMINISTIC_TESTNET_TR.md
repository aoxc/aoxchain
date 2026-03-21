# Deterministik 5 Düğümlü Test Ağı (Test Only)

Bu kurgu, **gerçek üretim anahtarı değil**, yalnızca geliştirme / demo / entegrasyon testi için hazırlanmış **kalıcı ve herkese açık** test seed'leri içerir.

## İçerik

`configs/deterministic-testnet/` altında şunlar üretilir:

- `accounts.json`: 5 bilinen düğüm, adres, validator kimliği, HD path ve seed listesi
- `genesis.json`: aynı 5 hesabı fonlayan genesis dosyası
- `nodes/*.toml`: düğüm başına p2p/rpc port planı
- `homes/<node>/identity/test-node-seed.hex`: düğümün deterministik seed dosyası
- `launch-testnet.sh`: her düğümü sırayla bootstrap edip blok ürettiren yardımcı script

## 5 sabit test düğümü

- atlas
- boreal
- cypher
- delta
- ember

## Üretim komutu

```bash
cargo run -q -p aoxcmd -- testnet-fixture-init \
  --output-dir configs/deterministic-testnet \
  --chain-num 77 \
  --fund-amount 2500000000000000000000
```

## Çalıştırma

```bash
bash configs/deterministic-testnet/launch-testnet.sh
```

## Güvenlik notu

Bu fixture içindeki seed'ler bilerek repoya yazılır. Bu nedenle:

- mainnet'te kullanılamaz,
- public testnet'te gerçek değer taşıyan hesap için kullanılamaz,
- sadece yerel / CI / demo ağları için kullanılmalıdır.
