# AOXC Artifacts — Production Readiness Bundle (v1)

Bu klasör, AOXC sisteminin **mainnet/testnet/devnet + production closure** süreçleri için
kanıt (evidence), denetim (audit), uyumluluk (compatibility) ve operasyonel kapanış
çıktılarını tek noktada toplar.

## Hedef

- CI/CD, release engineering, güvenlik ve SRE ekiplerinin aynı dosya ağacını kullanması.
- "Eksiksiz okuma" için standart bir giriş belgesi sağlanması.
- Her release artefact setinin doğrulanabilir ve izlenebilir olması.

## Dizin Yapısı

- `release-evidence/`
  - Build manifest, SBOM, provenance, compatibility matrix, audit raporu, checksum/signature kanıtları.
- `network-production-closure/`
  - Runtime status, telemetry snapshot, security drill, soak plan, rollout ve alarm kuralları.
- `index.json`
  - Bu klasördeki artefact grupları için makine-okunur envanter.

## Okuma Sırası (Reading Order)

1. `index.json`
2. `release-evidence/release-evidence-*.md`
3. `release-evidence/build-manifest-*.json`
4. `release-evidence/provenance-*.json`
5. `release-evidence/sbom-*.json`
6. `release-evidence/compat-matrix-*.json`
7. `network-production-closure/runtime-status.json`
8. `network-production-closure/telemetry-snapshot.json`
9. `network-production-closure/production-audit.json`
10. `network-production-closure/security-drill.json`

## Validasyon Kontrol Listesi

- Checksum dosyası mevcut ve binary hash ile uyumlu.
- Signature veya signature status dosyası mevcut.
- Provenance attestation mevcut.
- Compatibility matrix en az mainnet/testnet/devnet satırlarını içeriyor.
- Production audit sonucu `pass` veya eşdeğer onay durumunda.
- Runtime/telemetry snapshot dosyaları güncel release zaman penceresiyle uyumlu.

## Tavsiye Edilen Otomasyon

- Pre-release job:
  - build + test + sbom + provenance üretimi
- Release gate job:
  - signature/provenance/checksum doğrulama
- Post-release job:
  - production closure raporlarının `network-production-closure/` altına eklenmesi

## Not

Bu klasördeki belgeler, kriptografik sahiplik ispatının tek kaynağı değildir;
nihai doğrulama için imza doğrulama ve provenance doğrulaması birlikte çalıştırılmalıdır.
