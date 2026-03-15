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
