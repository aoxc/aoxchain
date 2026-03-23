# AOXC Gerçek Sürümleme ve Release Yol Haritası (TR)

Bu belge, AOXC için “alpha/beta hissiyatı veren geçici sürümleme” yerine **üretim odaklı**, **katman bazlı**, **gerçek ana ağ hedefiyle uyumlu** bir sürümleme modelini tanımlar.

Amaç:

- workspace sürümünü,
- protocol/core sürümünü,
- kernel sürümünü,
- block/vote/certificate format sürümlerini,
- testnet ve mainnet geçiş kapılarını

tek bir release dili altında birleştirmektir.

Bu belge özellikle şu soruya cevap verir:

> AOXC hangi sürüm numarasıyla hangi seviyede neyi garanti ediyor?

---

## 1. Temel ilke

AOXC için tek bir sayı yeterli değildir.

Çünkü sistemde farklı zamanlarda sabitlenecek katmanlar vardır:

1. **Binary / dağıtım sürümü**
2. **Canonical Core sürümü**
3. **Consensus Kernel sürümü**
4. **Block format sürümü**
5. **Vote / certificate format sürümü**
6. **Genesis authority sürümü**

Bu nedenle AOXC release modeli iki eksende çalışmalıdır:

- **ürün sürümü** (operator ve binary dünyası için),
- **protokol sürümü** (core/kernel uyumluluğu için).

---

## 2. Resmi adlandırma kararı

AOXC içinde iki merkezî çekirdek katman vardır:

### 2.1 AOXC Canonical Core

Bu katman `aoxcore` crate’idir.

Sorumlulukları:

- canonical block domain,
- canonical transaction domain,
- canonical receipts domain,
- identity / key-bundle truth,
- genesis truth,
- deterministic hash and encoding rules,
- future block assembly contracts.

### 2.2 AOXC Covenant Kernel

Bu katman `aoxcunity` crate’idir.

Sorumlulukları:

- consensus state machine,
- block admission,
- vote admission,
- fork choice,
- quorum,
- timeout / continuity,
- finality certificates,
- replay / recovery,
- evidence persistence.

### 2.3 Diğer workspaceler için ilke

Diğer crate’lerde ayrıca “ayrı kernel core” yaratılmaz.

Onun yerine her crate, kendi sorumluluğuna göre **kernel boundary** veya **core adapter** rolü taşır:

- `aoxcnet` → network boundary
- `aoxcdata` → persistence boundary
- `aoxcexec` → execution boundary
- `aoxcmd` → operator/runtime boundary
- `aoxcrpc` → rpc boundary
- `aoxcmob`, `aoxcsdk` → client boundary

Bu kararın nedeni şudur:

- tek core truth,
- tek kernel truth,
- çoklu boundary/integration surface.

Bu yapı, hem güvenlik hem de bakım için zorunludur.

---

## 3. Tavsiye edilen resmi sürümleme modeli

AOXC’de aşağıdaki sürüm katmanları birlikte tutulmalıdır.

### 3.1 Binary / ürün sürümü

Bu sürüm son kullanıcı, operator, release artifact ve binary adlandırması içindir.

Önerilen format:

`MAJOR.MINOR.PATCH-codename`

Örnek:

- `0.3.0-akdeniz`
- `0.6.0-marmara`
- `1.0.0-testnet`
- `2.0.0-mainnet`

Buradaki ek isim (codename), release ailesini ayırt etmek için yararlıdır; fakat asıl uyumluluk garantisi semantic version tarafından taşınır.

### 3.2 Protocol line sürümü

Canonical protocol uyumluluğu için ayrıca açık bir protokol etiketi tutulmalıdır.

Önerilen etiket:

- `AOXC-CORE-V1`
- `AOXC-CORE-V2`

Bu sürüm, `aoxcore` içindeki canonical veri sözleşmelerini kapsar.

### 3.3 Kernel line sürümü

Consensus state machine uyumluluğu için ayrıca kernel etiketi tutulmalıdır.

Önerilen etiket:

- `AOXC-COVENANT-KERNEL-V1`
- `AOXC-COVENANT-KERNEL-V2`

Bu sürüm, `aoxcunity` içindeki state transition semantiğini kapsar.

### 3.4 Format sürümleri

Ayrıca format seviyesinde ayrı sürümler tutulmalıdır:

- `AOXC-BLOCK-FMT-V1`
- `AOXC-VOTE-FMT-V1`
- `AOXC-CERT-FMT-V1`
- `AOXC-GENESIS-AUTH-V1`

Bu format etiketleri, binary sürüm yükselse bile protocol compatibility’yi net tutar.

---

## 4. “Alpha” dilinden çıkış kararı

AOXC hedefi üretim seviyesi olduğu için uzun süre “alpha” dilinde kalmak doğru değildir.

Ancak doğrudan `v1.0.0 mainnet` denmesi de teknik dürüstlüğü bozabilir.

Bu yüzden önerilen politika:

### Aşama A — Pre-Mainnet hardening

- Binary sürümler semver ile ilerler.
- “alpha” sözcüğü kaldırılır.
- Ama major sürüm `0.x` çizgisinde kalır.

Örnek:

- `0.3.0-akdeniz`
- `0.4.0-datca`
- `0.5.0-frigya`

### Aşama B — Public testnet commitment

İlk gerçek public/stable testnet açıldığında:

- binary major `1.x` olur,
- ama bu **mainnet** anlamına gelmez,
- bu “protocol stabilized public testnet line” anlamına gelir.

Örnek:

- `1.0.0-testnet`
- `1.1.0-testnet`
- `1.2.0-testnet`

### Aşama C — Mainnet constitutional release

İlk production-grade mainnet açıldığında:

- binary major `2.x` olur.

Örnek:

- `2.0.0-mainnet`

Bu karar bilinçlidir:

- `1.x` = public, stable, protocol-serious testnet line
- `2.x` = constitutional mainnet line

Bu ayrım, teknik dürüstlük ile üretim vizyonunu aynı anda korur.

---

## 5. Gerçek release hedefleri

Bu bölüm, “hangi sürümde ne garanti ediliyor?” sorusuna cevap verir.

## 5.1 `0.3.0-akdeniz`

### Release adı

**AOXC Akdeniz**

### Hedef seviyesi

Kernel ve core mimarisinin isim, sınır ve contract seviyesinde sabitlenmesi.

### Bu sürümde zorunlu olanlar

- AOXC Canonical Core / AOXC Covenant Kernel ayrımının resmileşmesi
- key-bundle sisteminin canonical authority kaynağı olarak sabitlenmesi
- kernel boundary dokümanının tamamlanması
- canonical block assembly planının başlangıç hali
- deterministic block proposal contract’ının tasarlanması

### Bu sürümde henüz zorunlu olmayanlar

- tam public testnet
- tam finality certificates
- tam distributed revocation
- tam multi-lane execution

### Anlamı

Bu sürüm, “AOXC artık mimarisini seçti” sürümüdür.

---

## 5.2 `0.4.0-datca`

### Release adı

**AOXC Datça**

### Hedef seviyesi

Canonical block formation Phase 1.

### Zorunlu hedefler

- `aoxcore` içinde canonical assembly contracts
- deterministic block assembly pipeline
- lane-aware body/commitment modeli
- richer block header commitment set
- runtime’ın assembler çıktısını kullanması
- replay-safe proposal determinism testleri

### Anlamı

Bu sürümde AOXC “blok üretmeye başlayan” değil,  
**canonical block üretim mantığını sabitleyen** sistem olur.

---

## 5.3 `0.5.0-frigya`

### Release adı

**AOXC Frigya**

### Hedef seviyesi

Kernel hardening Phase 1.

### Zorunlu hedefler

- authenticated vote model
- equivocation evidence
- authority root / validator set root
- eligibility-safe voting power
- stricter block admission order
- crash-safe evidence persistence

### Anlamı

Bu sürümde consensus kernel “scaffold” olmaktan çıkıp gerçek kernel karakteri kazanır.

---

## 5.4 `0.6.0-marmara`

### Release adı

**AOXC Marmara**

### Hedef seviyesi

Finality and persistence hardening.

### Zorunlu hedefler

- real quorum certificate
- continuity certificate
- constitutional/covenant sealing path
- replay + snapshot + recovery consistency
- fork-choice hardening
- degraded-mode safety tests

### Anlamı

Bu sürümde AOXC, “kernel iskeleti”nden “certificate-driven consensus engine”e yaklaşır.

---

## 5.5 `1.0.0-testnet`

### Release adı

**AOXC Public Testnet**

### Hedef seviyesi

İlk gerçek, public, stable, protocol-committed testnet.

### Zorunlu hedefler

- protocol line sabitlenmiş olmalı (`AOXC-CORE-V1`)
- kernel line sabitlenmiş olmalı (`AOXC-COVENANT-KERNEL-V1`)
- block/vote/certificate formats sabitlenmiş olmalı
- deterministic replay testleri geniş olmalı
- multi-node adversarial consensus testleri hazır olmalı
- operator/runbook ve network boundary yeterince oturmuş olmalı
- public testnet reset politikası dokümante edilmeli

### Anlamı

`1.x` hattı, “deneme sürümü” değil;  
**protokol ciddiyetine ulaşmış testnet hattı** anlamına gelir.

---

## 5.6 `1.1.x` – `1.3.x`

### Hedef seviyesi

Public testnet stabilization line.

### Bu aralıkta yapılacaklar

- economic/security hardening
- network admission hardening
- operator UX hardening
- rpc/indexer compatibility stabilization
- external proof lanes
- settlement and bridge compatibility
- production incident tooling

### Kural

Bu aralıkta protocol-breaking değişiklik minimum tutulmalı; mümkünse `V1` line korunmalıdır.

---

## 5.7 `2.0.0-mainnet`

### Release adı

**AOXC Mainnet Constitutional Release**

### Hedef seviyesi

İlk gerçek production mainnet.

### Zorunlu hedefler

- constitutional kernel finality line tamamlanmış olmalı
- recovery / journal / snapshot güvenilir olmalı
- validator eligibility ve authority root production-grade olmalı
- identity revocation ve rotation politikaları net olmalı
- block assembly / execution replay güvenliği oturmuş olmalı
- upgrade / rollback / emergency policy tanımlanmış olmalı

### Anlamı

Bu sürümde AOXC, “public testnet chain” olmaktan çıkar ve production constitutional chain haline gelir.

---

## 6. Hangi kod hangi sürümde kilitlenecek?

Bu tablo, kod sorumluluklarının hangi release band’inde stabilize olacağını gösterir.

### 6.1 `aoxcore`

#### `0.3.x`

- identity / key-bundle line sabitlenir
- genesis / hashing cleanup sabitlenir

#### `0.4.x`

- canonical block assembly contracts gelir
- block format zenginleşir

#### `1.0.x`

- `AOXC-CORE-V1` freeze edilir

### 6.2 `aoxcunity`

#### `0.3.x`

- kernel boundary dili sabitlenir

#### `0.5.x`

- authenticated vote + authority semantics gelir

#### `0.6.x`

- certificate and finality path oturur

#### `1.0.x`

- `AOXC-COVENANT-KERNEL-V1` freeze edilir

### 6.3 `aoxcmd`

#### `0.3.x` – `0.6.x`

- operator plane hızla gelişebilir
- fakat protocol truth üretmez
- core/kernel boundary tüketicisi olarak davranır

#### `1.0.x`

- public operator/testnet UX sabitlenir

### 6.4 `aoxcnet`, `aoxcdata`, `aoxcexec`, `aoxcrpc`

#### `0.x`

- boundary/adapters hızla değişebilir

#### `1.0.x`

- V1 core/kernel line ile uyumlu boundary contracts stabilize edilir

---

## 7. Sürüm yükseltme kuralları

### PATCH

PATCH yalnızca:

- bug fixes,
- panic removal,
- non-breaking audit improvements,
- docs/runbook fixes

için kullanılmalıdır.

### MINOR

MINOR şu durumlarda yükseltilmelidir:

- yeni operator capability,
- yeni boundary integration,
- backward-compatible new fields,
- new testnet features,
- stronger internal validations

### MAJOR

MAJOR şu durumlarda yükseltilmelidir:

- protocol line freeze / new protocol line,
- kernel state transition breaking change,
- block/vote/certificate canonical format kırılması,
- mainnet-class release transition.

---

## 8. Binary adı ve sürüm gösterimi

Operator tarafında binary adı önerisi:

- `aoxc`

Ama release gösterimi şu formatta olmalıdır:

- Binary version: `aoxc 0.3.0-akdeniz`
- Core line: `AOXC-CORE-V1`
- Kernel line: `AOXC-COVENANT-KERNEL-V1`

Örnek version çıktısı:

```text
aoxc 0.3.0-akdeniz
core-line: AOXC-CORE-V1
kernel-line: AOXC-COVENANT-KERNEL-V1-draft
block-format: AOXC-BLOCK-FMT-V1-draft
vote-format: AOXC-VOTE-FMT-V1-draft
```

Bu gösterim, hem operator hem de protocol debugging için çok değerlidir.

---

## 9. Önerilen resmi karar

Bugünden itibaren AOXC için önerilen resmi çizgi şudur:

### Ürün çizgisi

- `0.3.0-akdeniz` → kernel/core architecture lock
- `0.4.0-datca` → block formation phase 1
- `0.5.0-frigya` → vote/authority hardening
- `0.6.0-marmara` → finality/persistence hardening
- `1.0.0-testnet` → public stable testnet
- `2.0.0-mainnet` → constitutional mainnet

### Çekirdek adları

- `aoxcore` → **AOXC Canonical Core**
- `aoxcunity` → **AOXC Covenant Kernel**

### Protokol adları

- `AOXC-CORE-V1`
- `AOXC-COVENANT-KERNEL-V1`
- `AOXC-BLOCK-FMT-V1`
- `AOXC-VOTE-FMT-V1`
- `AOXC-CERT-FMT-V1`

---

## 10. Son söz

AOXC için en doğru yaklaşım:

- “alpha” kelimesine sonsuza kadar takılı kalmamak,
- ama teknik olarak hazır olmadan “mainnet” dememek,
- sürümlemeyi sadece cargo sayısı olarak görmemek,
- protocol/core/kernel/format katmanlarını ayrı ayrı isimlendirmek,
- release hedeflerini mimari olgunlukla eşlemek

olmalıdır.

Bu model, hem üretim ciddiyetini hem de mühendislik dürüstlüğünü korur.
