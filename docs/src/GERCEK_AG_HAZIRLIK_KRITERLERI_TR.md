# AOXChain Gerçek Ağ (Canlı Zincir) Hazırlık Kriterleri (TR)

Bu doküman, "çalışan bir demo" ile "operasyonel olarak gerçek zincir" arasındaki farkı netleştirmek için hazırlanmıştır.

## 1) Go/No-Go Özeti

- **Mevcut durum:** Güçlü bootstrap/smoke altyapısı mevcut, ancak repository sinyalleri **pre-mainnet** seviyesini işaret ediyor.
- **Karar:** Aşağıdaki zorunlu kapılar tamamlanmadan "tam gerçek zincir" etiketi verilmemeli.

## 2) "Gerçek Zincir" için Zorunlu Kapılar

1. **Canlı ağ doğrulaması (pre-mainnet ötesi):**
   - Ürün konumlandırması ve operasyonel statü çıktılarında production-ready ifadesiyle uyumlu olmalı.
2. **Sürekli çalışan node-daemon (tek blok demo değil):**
   - `produce-once` gibi tek-atımlık komutlar yardımcıdır; ana doğrulama sürekli çalışan node-runner ile yapılmalı.
3. **Loopback smoke yerine gerçek multi-node ağ testi:**
   - En az 3 node, farklı süreç/host ağ topolojisi, yeniden senkron ve jitter/latency koşulları.
4. **RPC ve transport güvenliği:**
   - Public yüzeylerde TLS/mTLS, kimlik doğrulama, rate-limit ve erişim politikası zorunlu.
5. **Snapshot/backup/restore operasyonu:**
   - Disk bozulması, node kaybı, rollback ve kurtarma için tatbik edilmiş runbook.
6. **Partition/restart/recovery senaryoları:**
   - Ağ bölünmesi ve yeniden birleşme sonrası deterministik finality ve state tutarlılığı.
7. **İmzalı release artifact ve doğrulama zinciri:**
   - Binary/protokol artifact imzası + fingerprint doğrulama adımları.
8. **Interop/security gate zorunlu enforcement:**
   - CI/CD ve release pipeline'da fail/pass kapısı olarak çalışmalı (opsiyonel rapor değil).

## 3) Kanıt Paketi (Release öncesi beklenen çıktı)

Her release adayında aşağıdaki kanıtlar tek bir "readiness bundle" içinde saklanmalı:

- Multi-node entegrasyon test raporu (3+ node, partition/restart/recovery dahil)
- RPC güvenlik konfigürasyonu (TLS/mTLS, authz/authn)
- Yedekleme + geri yükleme tatbikat logları
- İmzalı artifact + hash/fingerprint doğrulama kayıtları
- `interop-gate --enforce` ve zorunlu kalite kapıları sonuçları

## 4) Önerilen Uygulama Sırası

1. **Node-run servis modu + health/readiness endpointleri**
2. **3-node deterministic test harness (partition/rejoin senaryoları)**
3. **RPC hardening (TLS/mTLS + erişim politikası)**
4. **Snapshot/restore prosedürü + düzenli kurtarma tatbikatı**
5. **Signed release pipeline + provenance doğrulaması**
6. **CI/CD'de zorunlu gate enforcement**

## 5) Sonuç

AOXChain altyapısı güçlü ve doğru yönde ilerliyor; ancak "tam gerçek zincir" iddiası için teknik bileşen kadar operasyonel/supply-chain güvenlik zincirinin de kapanması gerekiyor.
