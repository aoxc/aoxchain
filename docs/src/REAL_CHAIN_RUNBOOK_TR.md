# AOXChain Sürekli Çalışım Runbook (TR)

Bu runbook, AOXChain'i **sürekli döngüde** çalıştırmak, terminal loglarını almak ve sağlık kontrollerini tekrar etmek için hazırlanmıştır.

> Not: Bu akış, local/pre-production operasyon içindir. Mainnet-grade gerçek zincir için multi-host, güvenlik, recovery ve audit kapıları ayrıca tamamlanmalıdır.

## 1) Hızlı Başlangıç

```bash
make real-chain-run-once
```

Bu komut:
- release binary paketler,
- testnet key bootstrap yapar,
- genesis oluşturur,
- node bootstrap yapar,
- bir cycle boyunca `produce-once` döngüsü + `network-smoke` probe çalıştırır,
- logları `logs/real-chain/` altında toplar.

## 2) Sürekli Çalıştırma (daemon loop)

```bash
make real-chain-run
```

Varsayılan davranış:
- cycle bazlı sonsuz döngü (`MAX_CYCLES=0`)
- her cycle'da block üretim turu ve network sağlık probe'u
- runtime log + health log ayrı dosyalara yazılır.

## 3) Log İzleme

```bash
make real-chain-tail
```

İzlenen dosyalar:
- `logs/real-chain/runtime.log`
- `logs/real-chain/health.log`

## 4) Örnek Ayarlamalar

```bash
MAX_CYCLES=3 ROUND_PER_CYCLE=120 SLEEP_MS=750 make real-chain-run
NETWORK_ROUNDS=50 NETWORK_TIMEOUT_MS=4000 NETWORK_PAUSE_MS=150 make real-chain-run
```

## 5) Tekil Sağlık Testi

```bash
make real-chain-health
```

Bu komut, `network-smoke` ile loopback TCP davranışını hızlıca doğrular.

## 6) Operasyonel Notlar

- AOXC_HOME varsayılanı: `./.aoxc-real`
- Runtime log: `logs/real-chain/runtime.log`
- Health log: `logs/real-chain/health.log`
- Bu akışta cycle hataları loglanır ve sonraki cycle devam eder.

