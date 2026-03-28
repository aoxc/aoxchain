# AOXChain Durum Özeti (2026-03-28)

Bu not, kalıcı testnet/mainnet başlatma sorusu için tek sayfalık operasyon özeti verir.

## Hızlı karar

- **Şu an kalıcı testnet/mainnet başlatımı önerilmez.**
- `aoxc mainnet-readiness --format json` çıktısı: `readiness_score=62`, `verdict=not-ready`.
- `aoxc testnet-readiness --format json` çıktısı: `readiness_score=62`, `verdict=not-ready`.

## Kritik açıklar (gate blocker)

Readiness çıktısında kapanması gereken ana blokajlar:

1. aktif profil hedef profile eşleşmiyor (`validation` çalışıyor),
2. JSON log zorunluluğu kapalı,
3. genesis materyali aktif AOXC home'da bulunmuyor,
4. node runtime state hazır değil,
5. operator key aktif durumda değil.

## Anahtar güvenliği değerlendirmesi

- Kodda özel anahtar düz metin yerine şifreli root-seed envelope modeli kullanılıyor.
- Ancak repo güvenlik bildirimi, tek başına test/derleme geçmesinin üretim güvenliği garantisi olmadığını açıkça söylüyor.
- Harici audit, incident tatbikatları, yedek/rotation/revocation süreçleri tamamlanmadan production riskli kabul ediliyor.

## Zincir çalışırken geliştirme (upgrade) seviyesi

- Mainnet checklist'te rollback prosedürü maddesi kapalı görünüyor.
- Buna rağmen real network validation runbook içinde "henüz tamamlanmış migration kanıtı yok" notu var.
- Sonuç: kesintisiz/sorunsuz canlı upgrade disiplini için pratik migration evidence daha da güçlendirilmeli.

## Operasyon önerisi (sırayla)

1. `aoxc production-bootstrap --profile testnet --password <...>`
2. `aoxc testnet-readiness --enforce --format json` -> 100/candidate hedefi
3. soak + incident + recovery + migration rehearsal kanıtlarını artifact olarak sabitle
4. aynı zincir için `--profile mainnet` ile tekrar bootstrap + readiness + enforce
5. sadece iki profile da candidate/100 olduktan sonra kalıcı ağ açılışı
