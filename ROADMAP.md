# AOXChain Unified Roadmap (Single Official Roadmap)

Bu doküman repodaki **tek resmi yol haritasıdır**.
Alt klasörlerde roadmap tutulmaz.

## Faz 0 — Stabilizasyon ve Temizlik
- [ ] Merge-conflict/dokümantasyon tutarsızlıklarının sıfırlanması
- [ ] README/READ hiyerarşisinin sadeleştirilmesi
- [ ] Kritik dizinlerin kapsam tanımının netleştirilmesi

## Faz 1 — Deterministik Çekirdek Sertleştirme
- [ ] `aoxcore` state-transition doğruluk testlerinin genişletilmesi
- [ ] `aoxcunity` finality/safety adversarial senaryoları
- [ ] Invariant testlerinin CI gate’e bağlanması

## Faz 2 — Multi-VM Conformance
- [ ] `aoxcexec` lane policy versioning
- [ ] `aoxcvm` cross-lane replay conformance corpus
- [ ] malformed input rejection matrix ve hata kod standardı

## Faz 3 — Network ve Data Dayanıklılığı
- [ ] `aoxcnet` partition/healing senaryoları
- [ ] `aoxcdata` snapshot/restore/corruption detect testleri
- [ ] servis SLO metriklerinin ölçüm pipeline’ı

## Faz 4 — Operator Plane Kurumsallaşma
- [ ] `aoxcmd` mutating action audit trail
- [ ] `aoxckit` key lifecycle runbook standardı
- [ ] `aoxchub` dashboard + alarm + incident akışı

## Faz 5 — Güvenlik ve Release Evidence
- [ ] SBOM/provenance/signature zincirinin release gate’e alınması
- [ ] threat model ve incident drill periyodu
- [ ] bağımsız güvenlik denetimi hazırlığı

## Faz 6 — Testnet -> Mainnet Geçiş Programı
- [ ] testnet kapanış kriterleri
- [ ] genesis freeze ve validator onboarding planı
- [ ] rollback ve emergency governance prosedürü

## Faz 7 — Mainnet Launch ve Sonrası
- [ ] launch window operasyonu
- [ ] ilk 90 gün güvenilirlik ve performans takibi
- [ ] protocol upgrade governance takvimi

## Ölçülebilir başarı kriterleri
- Deterministic replay pass oranı: %100 (kritik senaryolarda)
- Konsensüs güvenlik ihlali: 0
- Kritik incident MTTR hedefi: < 30 dk
- Release evidence completeness: %100

## Yönetişim notu
Bu roadmap üzerindeki her madde; sahibi, hedef tarihi, kanıt linki ve kapanış notu ile yönetilmelidir.
