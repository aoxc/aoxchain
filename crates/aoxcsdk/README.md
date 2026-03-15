# aoxcsdk

## Purpose

`aoxcsdk`, AOXChain ile entegre olacak uygulamalar ve servisler için SDK yüzeyi sağlar.

## Hedef Kullanım

- zincir istemcisi geliştirme,
- operasyon otomasyonu (CI/CD, sağlık kontrolleri, doğrulama akışları),
- AOXChain node ve RPC katmanları ile typed entegrasyon.

## SDK Tasarım İlkeleri

1. **Deterministik ve açık davranış**: sürpriz varsayılanlardan kaçınma.
2. **Typed hata yüzeyi**: entegratörün hatayı doğru sınıflandırabilmesi.
3. **Güvenlik odaklı entegrasyon**: anahtar/kimlik işlemlerinde net sınırlar.
4. **Dokümante örnekler**: hızlı başlangıç + doğrulanabilir kullanım senaryoları.

## Local Development

Repository root'tan:

```bash
cargo check -p aoxcsdk
cargo test -p aoxcsdk
```

## Workspace Entegrasyonları

- Node komut yüzeyi: [`../aoxcmd/README.md`](../aoxcmd/README.md)
- RPC katmanı: [`../aoxcrpc/README.md`](../aoxcrpc/README.md)
- Ağ güvenliği: [`../aoxcnet/README.md`](../aoxcnet/README.md)

## Üretim ve Risk Notu

SDK varlığı tek başına üretim güvenliği garantisi vermez. Canlı ortama geçmeden önce:
- bağımsız güvenlik denetimi,
- tehdit modeli doğrulaması,
- operasyonel runbook ve rollback planı,
- sürüm geçiş testleri
zorunlu değerlendirilmelidir.
