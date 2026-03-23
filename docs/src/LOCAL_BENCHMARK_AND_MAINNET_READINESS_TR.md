# Yerel Benchmark ve Mainnet Hazırlık Analizi

Bu doküman iki yeni komutu özetler:

- `mainnet-readiness`: mühendislik hazır oluş yüzdesi ve blokaj analizi
- `load-benchmark`: yerel sentetik yük altında blok/tx üretim ölçümü

## Hazırlık yüzdesi

```bash
cargo run -q -p aoxcmd -- mainnet-readiness
```

Bu çıktı:

- `readiness_percent`
- `grade`
- `hard_blockers`
- `partial_gaps`
- `recommendations`

alanlarını üretir.

## Yerel yük testi

```bash
cargo run -q -p aoxcmd -- load-benchmark \
  --home configs/deterministic-testnet/homes/atlas \
  --rounds 20 \
  --tx-per-block 40 \
  --payload-bytes 256 \
  --network-rounds 8 \
  --timeout-ms 2000
```

## Önemli not

Bu benchmark yalnızca **tek süreç / tek makine / yerel sentetik** ölçümdür. Mainnet gerçek performans garantisi olarak yorumlanmamalıdır.
