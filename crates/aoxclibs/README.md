# README.md

> Scope: `crates/aoxclibs`  
> Status: `Mainnet Readiness Program / Active`  
> Governance: `AOXC Release + Security + Operations`  
> License: `MIT`

Bu doküman, `crates/aoxclibs` klasörü için **kurumsal seviye işletim ve geliştirme standardını** tanımlar.
Amaç: içeriği yalnızca teknik olarak değil, aynı zamanda denetlenebilir, sürdürülebilir ve ana ağ (mainnet) hazırlığına uygun hale getirmek.

## 1) Öncelikli Referanslar

1. [Root README](../../README.md)
2. [Root READ](../../READ.md)
3. [Foundation Roadmap](../../ROADMAP.md)
4. İlgili klasördeki yerel teknik dosyalar (`*.toml`, `*.rs`, `*.json`, `*.yaml`)

## 2) Mainnet Seviyesi Hedefler

- **Deterministik Davranış:** Aynı girdi → aynı çıktı, platformdan bağımsız.
- **Sürüm Disiplini:** Konfigürasyon/arayüz değişimleri kontrollü ve izlenebilir.
- **Operasyonel Dayanıklılık:** Geri alma planı, runbook ve olay yönetimi hazır.
- **Güvenlik Temeli:** En az ayrıcalık, güvenli varsayılanlar, gizli veri hijyeni.
- **Denetlenebilirlik:** Değişiklik gerekçesi, test kanıtı ve release notları mevcut.

## 3) Klasör İçeriği Tamamlama Standardı

Bu kapsam altındaki her alt klasör için aşağıdaki eksikler kapatılmalıdır:

- [ ] Klasör amacı ve sınırları açık yazıldı.
- [ ] Dosya türleri ve sahiplik modeli belirlendi.
- [ ] Üretim etkisi (konsensüs/ağ/veri) notlandı.
- [ ] Hata senaryoları ve rollback yaklaşımı eklendi.
- [ ] Gözlemlenebilirlik gereksinimleri (log/metric/alert) tanımlandı.

## 4) Değişiklik Kalite Kapıları (Minimum)

- [ ] Dokümantasyon güncellemesi değişiklik ile birlikte yapıldı.
- [ ] Test kapsamı güncellendi (birim/integrasyon/gerekirse e2e).
- [ ] Statik kontroller ve derleme doğrulandı.
- [ ] Geriye dönük uyumluluk etkisi açıklandı.
- [ ] Operasyon ekibi için uygulama notu eklendi.

## 5) Güvenlik ve Uyum Notları

- Gizli anahtar/seed/token benzeri veriler repoya alınmamalıdır.
- Güvenlik kritik değişimler için ek tasarım notu veya risk kaydı zorunludur.
- Konfigürasyon varsayılanları üretim için güvenli olacak şekilde belirlenmelidir.

## 6) Operasyonel Runbook Beklentisi

Her kritik akış için şu başlıklar bulunmalıdır:

1. Hazırlık (pre-flight kontrol)
2. Uygulama adımları
3. Başarı metrikleri
4. Geri alma (rollback)
5. Olay eskalasyon kanalı

## 7) Done Tanımı (Definition of Done)

Aşağıdaki koşullar sağlanmadan değişiklik “tamamlandı” sayılmaz:

- [ ] Teknik uygulama ve dokümantasyon tutarlı.
- [ ] Test/kalite kontrolleri geçti.
- [ ] Mainnet etkisi değerlendirildi ve kayıt altına alındı.
- [ ] İnceleme notu (PR açıklaması) kanıtlarla tamamlandı.
