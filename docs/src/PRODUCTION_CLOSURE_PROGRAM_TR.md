# AOXC Production Closure Program

Bu belge, üretim seviyesine yaklaşmak için repo içinde hemen uygulanabilen kapanış paketini tanımlar.

## 1. Network production closure

Aşağıdaki eksikler tek komut akışına bağlandı:

- multi-host validation → `scripts/validation/multi_host_validation.sh`
- partition / fault injection → `scripts/validation/network_production_closure.sh --scenario partition`
- state sync / snapshot recovery → `scripts/validation/network_production_closure.sh --scenario recovery`
- soak test + telemetry baseline → `scripts/validation/network_production_closure.sh --scenario soak`

Üretilen artifact klasörleri:

- `artifacts/distributed-validation/`
- `artifacts/network-production-closure/`

## 2. Release / binary trust chain

Supply-chain kapanışı için release evidence jeneratörü eklendi:

- `scripts/release/generate_release_evidence.sh`

Bu akış şu çıktıları üretir:

- release checksum,
- build manifest,
- compatibility matrix,
- production audit,
- signature status,
- provenance attestation status,
- enforced release evidence raporu.

> Not: gerçek imza ve provenance üretimi için CI/CD tarafında `AOXC_SIGNING_CMD` ve `AOXC_PROVENANCE_CMD` bağlanmalıdır.

## 3. Node / protocol compatibility governance

`aoxc compat-matrix` artık şunları yayınlar:

- binary version ↔ protocol line eşlemesi,
- block / vote / certificate format enforcement,
- backward-compatibility policy,
- supported upgrade paths,
- release trust-chain zorunlulukları.

Bu çıktı release evidence paketine otomatik dahil edilir.

## 4. Release gate önerisi

Minimum gate:

```bash
cargo fmt --all --check
cargo test -p aoxcmd
./scripts/validation/network_production_closure.sh --scenario soak
./scripts/release/generate_release_evidence.sh
```

## 5. Kalan gerçek dünya işleri

Bu patch ağır blocker’ların tamamını fiziksel olarak kapatmaz; ancak aşağıdakileri repo-native hale getirir:

- kanıt formatı,
- otomasyon giriş noktaları,
- release gate iskeleti,
- compatibility enforcement görünürlüğü.

Gerçek kapanış için hâlâ gerekir:

1. ayrı host’larda gerçek run,
2. gerçek imza altyapısı,
3. gerçek provenance emitter,
4. uzun süreli soak raporları,
5. dashboard ve alarm backend entegrasyonu.
