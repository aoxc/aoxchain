# AOXChain için İleri Seviye Üretim Önerileri (Katkı Paketi)

Bu belge, mevcut alpha seviyesini üretime daha hızlı yaklaştırmak için doğrudan uygulanabilir önerileri önceliklendirir.

Detaylı consensus-kernel hardening planı için ayrıca bkz. [`AOXC_KERNEL_HARDENING_MASTER_PLAN_TR.md`](./AOXC_KERNEL_HARDENING_MASTER_PLAN_TR.md).
Ayrıca `aoxcunity` için deterministik motor, replay güvenliği ve readiness seviyesi ayrımını netleştiren yol haritası için bkz. [`AOXCUNITY_ENGINE_ROADMAP_TR.md`](./AOXCUNITY_ENGINE_ROADMAP_TR.md).

## 1) Konsensus Güvenliği

- Çatallanma (fork) simülasyonları: ağ bölünmesi, gecikme, tekrar birleşme senaryoları.
- Çifte-imza/equivocation tespiti için cezalandırma (slashing) test seti.
- Finality penceresi boyunca deterministik replay testleri.

## 2) Stake + Hazine Güvenceleri

- Invariant testleri: toplam arz, stake kilidi, hazine bakiyesi korunumu.
- Property-based test: rastgele stake/unstake/ceza akışlarında muhasebe doğruluğu.
- Geriye dönük mutabakat: blok sonu snapshot ile state tree checksum eşleştirmesi.

## 3) API ve Ağ Savunması

- RPC için zorunlu oran limiti profilleri (public/private/admin).
- mTLS sertifika rotasyonu ve iptal listesi (CRL/OCSP) doğrulama testleri.
- P2P discovery tarafında Sybil maliyeti artırıcı kontrol katmanları.

## 4) Operasyon ve Öz-İyileştirme

- `scripts/node_supervisor.sh`: yerel dağıtımlarda çökme sonrası otomatik yeniden başlatma.
- Healthcheck + restart sayaçları ile “fail-fast + controlled-restart” politikası.
- Planlı bakım pencerelerinde güvenli draining prosedürü.

## 5) Sürümleme ve Denetim Hazırlığı

- Her release için SBOM üretimi.
- Artifact imzalama + checksum doğrulama adımı.
- Denetçi için threat model, veri akışı diyagramı, saldırı yüzeyi matrisi.

## Önerilen Sıradaki Teknik İş

1. CI'ya `cargo audit` ve haftalık güvenlik taraması job'ı eklenmesi.
2. Konsensus fault-injection test iskeletinin açılması.
3. Stake/hazine invariant test modülünün oluşturulması.
4. Mainnet öncesi bağımsız dış denetim sözleşmesinin başlatılması.
