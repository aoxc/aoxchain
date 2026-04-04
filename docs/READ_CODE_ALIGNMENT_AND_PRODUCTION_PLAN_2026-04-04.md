# READ ↔ Kod Uyum Denetimi ve Üretim Planı (2026-04-04)

## 1) Yönetici Özeti

Bu denetimde `READ.md` içindeki teknik sözleşme ile depo gerçekliği karşılaştırıldı.

- **Mimari yüzey tanımı genel olarak uyumlu** (çekirdek crate/surface adları ve make gate hedefleri mevcut).
- **Üretim seviyesi için kritik boşluklar devam ediyor** (roadmap release blocker'ları açık, production checklist kapatılmamış).
- **Kalite kapısında somut hata yakalandı ve düzeltildi** (`aoxcrpc` içinde unused import).

Sonuç: Depo bu revizyon anında **production-ready değil**, ancak üretim hazırlığı için net bir kapanış planı mevcut.

---

## 2) READ ile Kod Uyum Kontrolü

### 2.1 Katman/sorumluluk yüzeyleri
`READ.md` içinde belirtilen katmanlar (`aoxcore`, `aoxcunity`, `aoxcvm`, `aoxcexec`, `aoxcenergy`, `aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`, `aoxcmd`, `aoxckit`, `aoxchub`) depo içinde crate olarak bulunuyor.

### 2.2 Readiness komut yüzeyi
`READ.md` ve `README.md` içinde belirtilen readiness hedefleri (`build`, `quality`, `audit`, `testnet-gate`, `testnet-readiness-gate`, `quantum-readiness-gate`) Makefile içinde tanımlı.

### 2.3 Üretim iddiası uyumu
- `README.md` depo durumunu **aktif geliştirme** ve **üretim garantisi yok** olarak tanımlıyor.
- `docs/PRODUCTION_READINESS_CHECKLIST.md` zorunlu üretim maddelerini kapatılmış göstermiyor.
- `ROADMAP.md` release blocker'ların açık olduğunu net yazıyor.

Bu üç kaynak birlikte değerlendirildiğinde, `READ.md` sözleşmesi ile gerçek repo durumu arasında “hedef var / closure yok” tipi bir uyumsuzluk var: mimari yön doğru, üretim kapanışı eksik.

---

## 3) Bu Revizyonda Yapılan Somut Düzeltmeler

1. **Kalite kapısı hatası düzeltildi**
   - `crates/aoxcrpc/src/grpc/server.rs` test modülündeki kullanılmayan import kaldırıldı.

2. **Tamamlanmış/tekrarlı READ temizliği**
   - `models/READ.md` silindi (aynı içerik `models/README.md` içinde mevcut).

3. **Önceki kısa değerlendirme notu kaldırıldı**
   - `docs/PRODUCTION_READINESS_REVIEW_2026-04-04.md` silinip bu kapsamlı plan/uyum dokümanı ile değiştirildi.

---

## 4) Üretim Seviyesine Çıkış Planı (Execution Plan)

## P0 — Zorunlu Kapanış (Release-blocking)
1. `make testnet-readiness-gate` komutunu tam PASS noktasına getirmek.
2. `ROADMAP.md` içindeki 5 release blocker'ı kapatmak.
3. `docs/PRODUCTION_READINESS_CHECKLIST.md` içindeki tüm zorunlu maddeleri evidence ile işaretlemek.
4. CI'da fail-closed gate: skip/warn ile release'e izin vermemek.

## P1 — Güvenceyi Derinleştirme
1. Adversarial + fuzz + reorg matrislerini default kalite akışına bağlamak.
2. Replay-ledger ve forensics çıktısını operatör doğrulanabilir artefact olarak üretmek.
3. Proof/finality verifier surface için deterministik test kapsamını CI zorunlu hale getirmek.

## P2 — Kurumsal Üretim Sertliği
1. SBOM + provenance + imza doğrulamasını release paketine bağlamak.
2. Incident response rehearsal çıktısını release evidence bundle’a eklemek.
3. Operatör runbook tatbikatlarını per-release zorunlu kontrol olarak sabitlemek.

---

## 5) Definition of Done (Production)

Aşağıdakiler birlikte sağlanmadan `Status: PRODUCTION_READY` beyanı yapılmamalı:

- tüm zorunlu gate komutları PASS,
- checklist maddeleri evidence ile kapalı,
- roadmap release blocker'ları kapalı,
- commit SHA ile bağlanan denetlenebilir artefact paketi mevcut.
