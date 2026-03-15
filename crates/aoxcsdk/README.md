# aoxcsdk

## Purpose

`aoxcsdk` provides the SDK-facing integration surface for applications and services that connect to AOXChain.

## Target Use Cases

- building chain clients,
- operational automation (CI/CD, health checks, validation workflows),
- typed integrations with AOXChain node and RPC layers.

## SDK Design Principles

1. **Deterministic, explicit behavior**: avoid hidden defaults.
2. **Typed error surface**: enable precise error classification for integrators.
3. **Security-oriented integration**: enforce clear boundaries around key and identity flows.
4. **Documented examples**: provide quick-start and verifiable usage patterns.
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

## Workspace Integrations

- Node command surface: [`../aoxcmd/README.md`](../aoxcmd/README.md)
- RPC layer: [`../aoxcrpc/README.md`](../aoxcrpc/README.md)
- Network security layer: [`../aoxcnet/README.md`](../aoxcnet/README.md)

## Production and Risk Note

The SDK alone is not a production security guarantee. Before going live, validate:
- independent security audits,
- threat model verification,
- operational runbook and rollback plan,
- version upgrade tests.
=======
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
