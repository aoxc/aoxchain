# CMB v1 (Constitutional Meta Block) Tasarım Taslağı

Bu doküman, AOXC zincirini klasik **transaction-centric block** modelinden
**constitutional multi-root commitment block** modeline evirmek için uygulanabilir
bir yol haritası sunar.

## 1) Hedef

CMB v1 hedefi, blokları yalnızca işlem kapsayıcısı olmaktan çıkarıp şu üçlüyü
native olarak zincire mühürleyen birimlere dönüştürmektir:

- **Intent** (niyet)
- **Proof** (kanıt/attestation)
- **Settlement** (lane bazlı sonuç)

## 2) Mevcut AOXC Tohumları (Neden Bu Tasarım Uyumlu)

- `aoxcore` tarafında lifecycle block yaklaşımı (`Active`, `Heartbeat`, `EpochPrune`) var.
- `aoxcunity` tarafında section-based blok ve `LaneCommitment` + `ExternalProof` modeli var.
- `aoxcvm` tarafında multi-lane vizyonu (EVM, Sui/Move, WASM, Cardano tarzı) var.

Bu nedenle CMB v1, mevcut mimariyi kırmadan genişletme odaklıdır.

## 3) Header Modeli: İki Katman

### 3.1 Consensus Header (çekirdek)

Ağ canlılığı ve finality için minimal alanlar:

- `version`
- `network_id`
- `parent_hash`
- `height`
- `era`
- `round`
- `timestamp`
- `proposer`
- `body_root`
- `finality_root`

### 3.2 Constitutional Header Extension (fark yaratan katman)

- `identity_root`
- `ai_root`
- `pq_root`
- `external_settlement_root`
- `policy_root`
- `time_seal_root`

> Kural: Header büyük veri taşımaz, yalnızca deterministic commitment root taşır.

## 4) Section Ailesi (Body)

Mevcut section-based tasarım korunur; yeni section aileleri eklenir:

- **ExecutionSection**
  - lane execution summary, tx_count, fee/gas commitment,
    input/output/receipt roots
- **IdentitySection**
  - validator identity snapshot root, session key root, revocation root,
    authority epoch proof
- **PostQuantumSection**
  - scheme metadata, signer set root, hybrid signature policy,
    migration epoch metadata
- **AISection**
  - AI request/response hash, inference attestation hash,
    confidence band commitment, human override flag
- **ExternalSettlementSection**
  - external finalized proof commitment, checkpoint record,
    settlement receipt commitment
- **ConstitutionalSection**
  - legitimacy / continuity / execution certificate hash,
    constitutional seal hash
- **TimeSealSection**
  - valid-from / valid-until,
    delayed-effect ve epoch-bound action commitments

## 5) Block Tipi Evrimi

Aşırı şişen enum yerine tek blok + capability flags yaklaşımı önerilir:

- `EXECUTION_FLAG`
- `HEARTBEAT_FLAG`
- `SETTLEMENT_FLAG`
- `AI_ATTESTATION_FLAG`
- `PQ_ROTATION_FLAG`
- `CONSTITUTIONAL_FLAG`
- `RECOVERY_FLAG`

Böylece bir blok aynı anda birden çok rol taşıyabilir.

## 6) Multi-Root Modeli

CMB v1 ile dual-root yaklaşımından multi-root yaklaşımına geçilir:

- `intent_root`
- `execution_root`
- `receipt_root`
- `identity_root`
- `ai_root`
- `pq_root`
- `external_root`
- `policy_root`
- `finality_root`

Bu model light-client, audit ve domain-bazlı doğrulama yollarını ayrıştırır.

## 7) AI ve PQ İçin Altın Kurallar

### AI

- Tam prompt/response zincire yazılmaz.
- Zincire yalnızca denetlenebilir commitment izi yazılır.

### Post-Quantum

- Sadece klasik imzaya PQ eklemek yeterli değildir.
- Blokta aktif imza politikası ve geçiş dönemi açıkça commit edilmelidir:
  - `classical-only`
  - `hybrid`
  - `pq-preferred`
  - `pq-mandatory`
- `crypto_epoch` ile kademeli geçiş yapılır.

## 8) Uygulama Planı (Aşamalı)

### Faz 1 — Non-breaking hazırlık

- `aoxcunity` header’a extension root alanları ekle (varsayılan sıfır-root kabul).
- Yeni section türlerini feature-gated ekle.
- Hash domain ayrımlarını yeni alanlar için genişlet.

### Faz 2 — Semantik doğrulama

- Section arası invariant kontrollerini ekle (ör. AI section varsa policy root zorunlu).
- Capability flag doğrulayıcılarını builder + validator katmanına ekle.

### Faz 3 — Konsensüs entegrasyonu

- Proposal/vote doğrulamasında capability + root bütünlüğü kontrolü.
- Constitutional seal doğrulamasını finality akışına bağla.

### Faz 4 — Migration

- `crypto_epoch` tabanlı imza politikası geçiş takvimi yayınla.
- Eski block okuyucular için backward-compatible parse/verify stratejisi uygula.

## 9) Kaçınılması Gerekenler

- Blok içine tam AI çıktı/prompt gömmek.
- Blok içine büyük proof blob yazmak.
- Aşırı geniş block type enum tasarlamak.
- Execution/consensus sorumluluklarını bulanıklaştırmak.

## 10) Konumlandırma Cümlesi

> AOXC is not merely a transaction chain.
> AOXC is a constitutional commitment chain for identity, AI-governed actions,
> post-quantum authority, and multi-lane settlement.

Bu doküman ürün kimliği değil, uygulanabilir teknik çerçeve olarak kullanılmalıdır.
