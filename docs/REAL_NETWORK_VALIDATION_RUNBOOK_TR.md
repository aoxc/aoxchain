# AOXC Gerçek Ağ Doğrulama ve Operasyon Runbook'u

Bu doküman, istenen sırayı izler:

1. gerçek ağ doğrulaması
2. dayanıklılık / partition / fault
3. state sync ve recovery
4. soak test
5. telemetri
6. readiness kanıtı güncellemesi
7. upgrade / migration
8. operasyon runbook

---

## 1. Multi-host gerçek ağ testi

### Amaç
3-5 node'u ayrı hostlarda çalıştırıp gerçek peer-to-peer davranışı ve blok/tx yayılımını ölçmek.

### Kurulum
- her hostta aynı AOXC binary
- `configs/deterministic-testnet/` fixture dağıtımı
- host listesi: `configs/deterministic-testnet/hosts.txt`
- çalıştırma scripti: `scripts/validation/multi_host_validation.sh`

### Test adımları
- fixture'ı hostlara kopyala
- her hostta ilgili node home ile `node-bootstrap` çalıştır
- kısa dağıtık `node-run` çalıştır
- logları ve json çıktıları `artifacts/distributed-validation/` altında topla

### Gözlem
- peer bağlantısı var mı?
- blok yüksekliği hostlar arasında yakınsıyor mu?
- propagation gecikmesi ne kadar?

### Sorunlar
- peer discovery eksikliği
- gerçek cross-host propagation metriği eksikliği
- block dissemination için daha zengin network katmanı ihtiyacı

### Sonuç
Bu aşama tamamlandı sayılmadan readiness artmamalı.

### Sonraki adım
partition/fault senaryolarına geç.

### Genel not
Mevcut repo local fixture sunuyor; gerçek multi-host kanıtı ayrıca toplanmalı.

---

## 2. Partition ve fault senaryoları

### Amaç
Bozulma anındaki davranışı görmek.

### Kurulum
- 3+ host
- ssh erişimi
- mümkünse `tc netem`, firewall veya process control

### Test adımları
- host A ile B/C arasında partition oluştur
- bir node'u kill et ve yeniden başlat
- gecikme ekle
- paket kaybı / timeout uygula
- mümkünse fault injection ile bozuk mesaj dene

### Gözlem
- zincir duruyor mu, çatallanıyor mu, toparlanıyor mu?
- quorum davranışı beklendiği gibi mi?

### Sorunlar
Başarısızlık varsa neden ve tekrar üretim adımı yazılmalı.

### Sonuç
Her senaryo için beklenen/gözlenen davranış ayrı raporlanmalı.

### Sonraki adım
state sync ve recovery testine geç.

### Genel not
Normal çalışma tek başına yeterli kabul edilmemeli.

---

## 3. State sync ve snapshot recovery

### Amaç
Düşen veya yeni gelen node'un state'i güvenli ve tutarlı şekilde alabilmesi.

### Kurulum
- snapshot formatı belirlenmeli
- restore akışı tanımlanmalı
- recovery sırasında hash/height doğrulaması yapılmalı

### Test adımları
- çalışan ağdan snapshot al
- boş node'da restore et
- düşen node'u geri getir
- height/state hash tutarlılığını kontrol et

### Gözlem
- recovery süresi
- veri tutarlılığı
- replay veya veri kaybı var mı?

### Sorunlar
Mevcut repo içinde bu alan henüz eksik kabul edilmeli.

### Sonuç
Mainnet öncesi blocker.

### Sonraki adım
soak test.

### Genel not
Recovery kanıtı olmadan production iddiası risklidir.

---

## 4. Soak test

### Amaç
Uzun süreli kararlılığı görmek.

### Kurulum
- uzun süre çalışan node seti
- log döndürme
- CPU / memory gözlemi

### Test adımları
- belirli süre sürekli blok üret
- tx akışı ver
- kaynak tüketimini kaydet

### Gözlem
- stall, drift, deadlock, memory büyümesi

### Sorunlar
uyarılar ve hata özeti rapora eklenmeli.

### Sonuç
Kısa benchmark yerine uzun dönem davranış görünür olur.

### Sonraki adım
telemetri ve alarmlar.

### Genel not
Soak yoksa gerçek operasyonel güven oluşmaz.

---

## 5. Gözlemlenebilirlik ve telemetri

### Amaç
Çalışma ve bozulma sebeplerini görünür yapmak.

### Kurulum
Toplanması beklenen metrikler:
- health
- block time
- tx throughput
- peer count
- sync state
- error count

### Test adımları
- metrikleri üret
- raporla veya dashboard'a aktar
- kritik alarm eşiklerini belirle

### Gözlem
hangi metrik nereden geliyor ve ne kadar güvenilir?

### Sorunlar
peer count / sync state / propagation görünürlüğü halen zayıf olabilir.

### Sonuç
telemetri standardı yazılmalı.

### Sonraki adım
kanıta dayalı readiness güncelle.

### Genel not
Sadece çalışıyor demek yeterli değildir.

---

## 6. Readiness skorunu gerçek kriterlerle güncelle

### Amaç
Skoru kanıta bağlamak.

### Kurulum
- model dosyası: `models/mainnet_readiness_evidence_v1.yaml`
- komut: `aoxcmd mainnet-readiness --evidence <file>`

### Test adımları
- her kriter için done tanımı yaz
- evidence alanını doldur
- blockers ve stretch goals'u ayır

### Gözlem
skor neden arttı/azaldı açıkça görünmeli.

### Sorunlar
kanıt yoksa skor artmamalı.

### Sonuç
hazır oluş yüzdesi daha dürüst hale gelir.

### Sonraki adım
upgrade / migration.

### Genel not
komut çıktısı tek başına değil, evidence ile anlamlıdır.

---

## 7. Upgrade ve migration hazırlığı

### Amaç
State veya runtime değiştiğinde güvenli geçiş planı oluşturmak.

### Kurulum
- versioning planı
- schema değişim notu
- rollback planı

### Test adımları
- eski veriden yeni sürüme geçiş dene
- başarısız migration senaryosu düşün
- rollback testi yap

### Gözlem
hangi veri riskli, hangi adım geri alınabilir?

### Sorunlar
bu repo içinde henüz tamamlanmış migration kanıtı yok.

### Sonuç
mainnet öncesi planlanmalı.

### Sonraki adım
operator checklist tamamla.

### Genel not
sürüm geçişi hazırlıksız bırakılmamalı.

---

## 8. Operasyon runbook'u

### Amaç
Sistemi yönetilebilir hale getirmek.

### Kurulum
operator checklist tutulmalı.

### Test adımları
- node başlatma
- node durdurma
- yeniden senkronizasyon
- snapshot alma/yükleme
- peer sorunu giderme
- incident başlangıç akışı

### Gözlem
hangi adımlar manuel, hangileri otomatik?

### Sorunlar
snapshot/state sync eksikleri bu bölümü de etkiler.

### Sonuç
runbook operasyonel kabul kriteridir.

### Sonraki adım
kanıtları tamamlayıp readiness skorunu güncelle.

### Genel not
runbook olmadan sistem yönetilebilir sayılmaz.
