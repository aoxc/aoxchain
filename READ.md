# AOXChain — Advanced Omnichain Execution Chain (Canonical Definition)

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="220" />
</p>

> Status: `Engineering Mainnet Program`  
> Domain: `Deterministic L1 + Multi-VM Execution + Operator-Grade Control Plane`  
> Canonical Scope: `This document is the single technical definition of the chain at repository level.`

---

## 1) AOXChain nedir?

AOXChain; deterministik state-transition çekirdeği, modüler yürütme motoru, servis katmanı ve operasyon düzlemini ayrıştıran kurumsal seviye bir zincir mimarisidir.

Temel hedef:
- güvenli finalite,
- tekrar üretilebilir (replay-stable) yürütme,
- ölçülebilir operasyon kalitesi,
- kontrollü protokol evrimi.

---

## 2) Mimari katmanlar (single source of truth)

### 2.1 Kernel (konsensüs-kritik alan)
- `crates/aoxcore`: blok/tx/state/receipt gibi çekirdek veri modeli ve state-transition temelleri.
- `crates/aoxcunity`: konsensüs, finality, quorum ve safety kuralları.

**Kural:** Kanonik zincir durumunu sadece kernel kararları değiştirebilir.

### 2.2 Execution plane (VM ve lane orkestrasyonu)
- `crates/aoxcexec`: deterministik yürütme politikaları, lane envelope, execution accounting.
- `crates/aoxcvm`: çoklu yürütme lane yönlendirmesi (native / EVM / WASM / uyumluluk lane’leri).
- `crates/aoxcenergy`: maliyetlendirme/gas/ekonomi kuralları.

**Kural:** Aynı kanonik girdi için aynı yürütme sonucu üretilmelidir.

### 2.3 System services
- `crates/aoxcnet`: p2p, gossip, discovery, sync.
- `crates/aoxcrpc`: RPC/API yüzeyi.
- `crates/aoxcdata`: persistence, index, data lifecycle.
- `crates/aoxconfig`: tip güvenli konfigürasyon ve doğrulama.

### 2.4 Operator plane
- `crates/aoxcmd`: CLI operasyon yönetimi.
- `crates/aoxckit`: key/crypto araçları.
- `crates/aoxchub`: desktop control-plane arayüzü.

**Kural:** UI/CLI konsensüs otoritesi değildir; yalnızca kontrol ve gözlemlenebilirlik yüzeyidir.

---

## 3) Zincirin deterministik çalışma sözleşmesi

1. Non-deterministic girdiler normalize edilmeden kernel’e etkide bulunamaz.
2. Malformed/policy-invalid payload’lar state mutation öncesi reddedilir.
3. Policy değişimleri versiyonlu ve aktivasyon kapsamı tanımlı yapılır.
4. Release iddiaları commit’e bağlı test/evidence paketiyle doğrulanır.
5. Belirsizlikte sistem fail-closed davranır.

---

## 4) Mainnet kalite kapıları

Bir sürüm “mainnet adayı” sayılmadan önce:

- Deterministic replay testleri (çoklu lane) geçmeli,
- Konsensüs ve ağ dayanıklılık senaryoları raporlanmalı,
- Snapshot/restore bütünlük testi kanıtlanmalı,
- API ve config uyumluluk etkisi açıklanmalı,
- Güvenlik ve operasyon runbook’ları güncel olmalı,
- Release artifact zinciri (binary hash, sbom, provenance, signatures) tamamlanmalı.

---

## 5) Konfigürasyon ve ortam modeli

`configs/` altında localnet, devnet, testnet, validation, mainnet ve sovereign template ortamları bulunur.
Her ortam için profile/genesis/validator/release-policy seti sürümlü tutulur.

**İlke:** Ortamlar arası farklar açık olmalı; gizli varsayım bırakılmamalı.

---

## 6) Güvenlik, anahtar ve denetlenebilirlik

- Anahtar yaşam döngüsü (üretim, saklama, rotasyon, iptal) denetlenebilir olmalıdır.
- Konsensüs-kritik kod değişimleri risk notu ile merge edilir.
- Olay müdahalesi ölçülebilir ve runbook tabanlı yürütülür.
- Operator action -> evidence mapping kırılmamalıdır.

---

## 7) Geliştirme prensipleri

- Minimal, açık, testlenebilir değişiklik.
- Dokümantasyon ve kod birlikte güncellenir.
- Geriye dönük uyumluluk etkisi açıkça yazılır.
- Klasör READ/README dosyaları yalnızca “kodun ne yaptığı”nı anlatır; roadmap içermez.

---

## 8) Resmi referanslar

- Tek roadmap: [ROADMAP.md](./ROADMAP.md)
- Lisans: [LICENSE](./LICENSE)
- Mimari: `docs/ARCHITECTURE.md`
- Execution modeli: `docs/EXECUTION_MODEL.md`
- İnvariantlar: `docs/SYSTEM_INVARIANTS.md`
