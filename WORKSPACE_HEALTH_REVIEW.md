# AOXChain Workspace & Runtime Full Development Plan (2026-04-12)

## Purpose

Bu doküman şu hedefe hizmet eder: AOXChain workspace ve runtime yüzeyini **dev / testnet / mainnet** için eksiksiz, doğrulanabilir ve operasyonel olarak güvenilir hale getirmek.

> Not: "hatasız" mutlak garanti olarak teknik olarak ispatlanamaz; hedef, **fail-closed + kanıt tabanlı + sürekli gate** yaklaşımıyla üretim riskini sistematik olarak minimize etmektir.

---

## 1) Current Verified State

### 1.1 Workspace integrity

- Workspace tanımı 19 üyeyi içeriyor.
- Yerel doğrulama: 19 üyenin tamamı diskte mevcut ve workspace’e bağlı.
- Sonuç: orphan crate / eksik üyelik tespit edilmedi.

### 1.2 Build/compile surface

- `cargo check --workspace --exclude aoxchub --all-targets` başarılı.
- `cargo check -p aoxchub --all-targets` başarılı.
- Sonuç: check seviyesinde tüm workspace derlenebilir durumda.

### 1.3 Runtime model

- Runtime mevcut.
- Operasyon modeli tek-path/single-runtime sözleşmesi ile kurgulanmış.
- Yapı sabit (runtime path contract), kaynak seçimi kontrollü (network/profile tabanlı).

---

## 2) Runtime Contract: Fixed vs Configurable

## 2.1 Structural contract (fixed)

Sabit kalan temel sözleşme:

- tek runtime kökü,
- canonical runtime dizinleri (identity/state/config/db/operator/snapshots),
- evidence ve audit dosya yolları,
- runtime lifecycle adımları (`source-check`, `install`, `verify`, `activate`, `status`, `doctor`).

## 2.2 Environment contract (configurable)

Konfigüre edilebilir yüzey:

- `AOXC_NETWORK_KIND` seçimi (`dev`, `testnet`, `mainnet`),
- profile ve release-policy girdileri,
- validator/topoloji parametreleri,
- rollout stratejisi (canary/staged/progressive).

Yani: **altyapı sözleşmesi sabit**, **çalıştırma bağlamı kontrollü değişken**.

---

## 3) Full Runtime Targets by Environment

## 3.1 Dev Runtime (fast feedback, fail-fast)

Zorunlu hedefler:

1. Her PR’da tam kalite kapısı (fmt, clippy, test, check).
2. Runtime doctor + smoke komutlarının otomatik doğrulanması.
3. Snapshot üretim/geri-yükleme doğrulaması.
4. Konsensus/ledger telemetri alanlarının schema uyumluluğu.
5. Deterministic replay mini-suite (seed’li, tekrar üretilebilir).

Başarı kriteri:

- Dev branch üzerinde merge öncesi tüm gate’ler yeşil,
- en son evidence bundle erişilebilir,
- failure durumunda rollback komutları dokümante ve testli.

## 3.2 Testnet Runtime (soak + migration proof)

Zorunlu hedefler:

1. Çok düğümlü uzun süreli dayanım testi (soak).
2. Genesis/validator/bootnode/certificate doğrulama zinciri.
3. Upgrade/downgrade/migration tatbikatı (policy uyumlu).
4. Fault-injection senaryoları (network partition, stale state, replay attempt).
5. Readiness raporu + remediation çıktıları arşivlenmeli.

Başarı kriteri:

- testnet gate + readiness gate her sürüm adayı için kanıtlı,
- deterministic convergence ihlali yok,
- recovery drill raporu güncel.

## 3.3 Mainnet Runtime (change control + safety-first)

Zorunlu hedefler:

1. İmzalı release artifact + checksum + provenance.
2. İki aşamalı aktivasyon (preflight verify -> controlled activate).
3. Canary validator grubu ile kademeli yayılım.
4. SLO/SLI odaklı runtime gözlemi (finality latency, liveness, error budget).
5. Incident playbook ve zorunlu postmortem akışı.

Başarı kriteri:

- release promotion yalnızca evidence-complete paket ile,
- policy-root / scheme / replay kontrolleri tam uyumlu,
- acil geri dönüş (rollback) süresi ölçülmüş ve periyodik testli.

---

## 4) Missing Pieces to Reach "Full"

Aşağıdakiler tamamlanmadan "eksiksiz runtime" iddiası zayıf kalır:

1. **Environment-specific runtime stability policy**
   - Hangi dosya/alan immutable, hangisi mutable açıkça tanımlanmalı.

2. **Mandatory evidence publishing discipline**
   - Her gate koşusunun artifact’i değiştirilemez depoya alınmalı.

3. **Deterministic replay regression matrix**
   - kritik transaction sınıfları için sabit seed + golden output saklanmalı.

4. **Snapshot compatibility policy**
   - runtime/state snapshot’ları için versioned compatibility kuralı zorunlu olmalı.

5. **Operational SLO ownership**
   - dev/testnet/mainnet için net eşikler ve on-call sorumluluk matrisi belirlenmeli.

---

## 5) Full Implementation Roadmap

## Phase A — Immediate (0-2 weeks)

- CI pipeline’da zorunlu gate zinciri aktif et:
  - `make quality`
  - `make audit`
  - `make testnet-gate`
  - `make testnet-readiness-gate`
- Evidence bundle standardı tanımla (manifest + checksum + retention).
- Runtime doctor komutları için otomatik smoke stage ekle.

## Phase B — Hardening (2-6 weeks)

- Deterministic replay + snapshot compatibility test seti ekle.
- Testnet soak/fault-injection pipeline’ını nightly zorunlu hale getir.
- Release signing + verify prosedürünü promotion gate’e bağla.

## Phase C — Mainnet-grade operations (6+ weeks)

- Canary rollout standardı ve stop-the-line koşullarını aktif et.
- SLO/SLI dashboard + alert policy + runbook bağını üretimleştir.
- Recovery/rollback tatbikatlarını periyodik takvime bağla.

## 5.1 Executable closure commands (implemented)

Bu plan artık doğrudan çalıştırılabilir toplu hedeflerle desteklenir:

- `make dev-full`
- `make testnet-full`
- `make mainnet-full`
- `make full-runtime-all`

Komut akışı:

- `dev-full`: quality + dev runtime source/activate + runtime doctor + phase1 determinism seti
- `testnet-full`: dev-full + testnet gate + testnet readiness gate
- `mainnet-full`: testnet-full + network identity gate + production-full
- `full-runtime-all`: dev -> testnet -> mainnet zinciri

---

## 6) Direct Answers (Requested)

- **"Full geliştirme" var mı?**
  - Güçlü temel var; ancak üretim-grade "full" için yukarıdaki Phase A/B/C kapanışları zorunlu.

- **"Runtime var mı?"**
  - Evet, runtime hem Makefile lifecycle hem de `aoxcmd` runtime telemetry/persist akışı ile mevcut.

- **"Runtime sabit mi?"**
  - Yapısal sözleşme sabit; environment/profile seçimi kontrollü değişken.

- **"Eksik var mı?"**
  - Derleme bloklayıcısı görünmüyor; ana eksikler operasyonel kanıt disiplini, replay/snapshot rejimi ve environment-specific stabilite politikası.

- **"Dev/Testnet/Mainnet tam mı?"**
  - Bu sürümde toplu closure hedefleri (`dev-full`, `testnet-full`, `mainnet-full`, `full-runtime-all`) eklendi; tamlık için CI’da merge-blocker olarak zorunlu çalıştırılmalı.

---

## 7) Recommended Next Action (Single Best Move)

Tek en yüksek etkili adım:

**Dev/Testnet/Mainnet için evidence zorunluluğunu CI’da "merge blocker" yap**
(quality + audit + readiness + signed artifact verify).

Bunu yaptığında runtime olgunluğu ölçülebilir, denetlenebilir ve sürdürülebilir hale gelir.
