# AOXChain Security and Risk Notice

This document provides a concise risk framework for responsible evaluation and operation of the AOXChain codebase.

## 1) Important Warning

AOXChain is under active development. Successful compilation, passing tests, or working local smoke commands do **not**
by themselves guarantee economic security, adversarial resilience, or regulatory compliance.

## 2) Direct Copy/Fork Risk

It is high-risk to move this project directly into production before completing the following:
- independent third-party security audits,
- economic/incentive attack simulations,
- incident response practice for node operations,
- backup, key rotation, certificate revocation, and disaster recovery procedures.

## 3) Minimum Security Checklist

1. **Code quality gates**
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
2. **Configuration hygiene**
   - isolated dev/test/mainnet environments,
   - secure storage for secrets and key material.
3. **Operational resilience**
   - centralized logs and actionable alert thresholds,
   - backup and restore drills.
4. **Release process controls**
   - signed release artifacts,
   - tested rollback plans.

## 4) About the "%99.99 secure" Goal

Targets such as "99.99% secure" are intent statements, not absolute guarantees.
In practice, security is achieved through continuous auditing, testing, monitoring, and rapid incident response.

## 5) Recommended Next Steps

- commission a formal external security audit,
- maintain threat modeling as a living document,
- run regular chaos scenarios for partition/replay/DoS behavior,
- establish a responsible disclosure process for vulnerabilities.

# AOXChain Güvenlik ve Risk Bildirimi (TR)

Bu doküman, AOXChain kod tabanının sorumlu şekilde değerlendirilmesi için kısa bir risk çerçevesi sunar.

## 1) Önemli Uyarı

AOXChain aktif geliştirme aşamasındadır. Kodun derlenmesi, testlerin geçmesi veya local smoke komutlarının çalışması;
tek başına ekonomik güvenlik, adversarial dayanıklılık veya regülasyon uyumluluğu garantisi vermez.

## 2) Doğrudan Kopyalama/Forklama Riski

Aşağıdaki kalemler tamamlanmadan projeyi doğrudan üretime taşımak yüksek risklidir:
- bağımsız üçüncü taraf güvenlik denetimi,
- zincir ekonomisi ve teşvik modelinin saldırı simülasyonları,
- node operasyonu için olay müdahale (incident response) pratikleri,
- yedekleme, anahtar döndürme, sertifika iptal ve felaket kurtarma prosedürleri.

## 3) Minimum Güvenlik Kontrol Listesi

1. **Kod Kalitesi**
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
2. **Konfigürasyon Hijyeni**
   - Ayrı ortamlar: dev/test/mainnet
   - Gizli veriler için güvenli saklama
3. **Operasyonel Dayanıklılık**
   - Log toplama ve alarm eşikleri
   - Yedekleme + geri yükleme tatbikatı
4. **Yayın Süreci**
   - İmzalı release süreci
   - Geri alma (rollback) planı

## 4) Hedef Güvenlik Seviyesi Hakkında

"%99.99 güvenli" gibi hedefler bir niyet göstergesidir; mutlak güvenlik garantisi değildir.
Pratikte güvenlik, sürekli denetim + test + izleme + hızlı müdahale disiplinlerinin birleşimidir.

## 5) Önerilen Sonraki Adımlar

- Harici audit firması ile formal güvenlik denetimi,
- Tehdit modelini yaşayan doküman olarak sürdürme,
- Ağ bölünmesi/replay/doS senaryoları için düzenli chaos testleri,
- Güvenlik açıkları için sorumlu açıklama (responsible disclosure) süreci.
