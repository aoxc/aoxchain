# AOXChain Tek Etap Tamamlama Programı (TR)

Bu belge, AOXChain için önceki çok-etaplı yaklaşımı sadeleştirip **tek etapta tamamlama programı** olarak yeniden tanımlar.

Yeni karar şudur:

- ayrı ayrı 9 etap ilerlemek yerine,
- `v0.1.1-alpha` başlangıç bazını alıp,
- dokümantasyon, mdBook, sürümleme, altyapı dosyaları, network, consensus, recovery, RPC security, observability, release ve launch gate alanlarını
- **tek büyük kapanış etabı** içinde eksiksiz hale getirmek.

Bu yüzden bu belgedeki ana hedef artık şudur:

> **Amaç: altyapı %100.**

Bu ifade soyut bir slogan değildir. Bu program içinde “altyapı %100” şu anlama gelir:

1. eksik docs kalmaması,
2. eksik mdBook gezinti veya kırık referans kalmaması,
3. eksik altyapı dosyası / klasör / örnek config / artifact yeri belirsizliği kalmaması,
4. testnet ile mainnet arasındaki farkların yazılı ve ölçülebilir olması,
5. testnet ve mainnet için gerekli teknik kapıların aynı program altında kapanması,
6. release, rollback, upgrade, security ve operasyon konularının belge dışına taşmaması.

---

## 1. Program sürümü

### Başlangıç etiketi
- **Program başlangıç sürümü:** `v0.1.1-alpha`
- **Program tipi:** Tek etap tamamlama
- **Program hedefi:** testnet + mainnet hizalı altyapı kapanışı
- **Başlangıç ilkesi:** parça parça ilerleme değil, kapanış odaklı tamamlama

### Sürüm mantığı
Bu belgede `v0.1.1-alpha` şu amaçla kullanılır:

- eksiklerin görünür hale getirilmesi,
- başlangıç kapsamının dondurulması,
- tamamlanacak dosya/klasör/doc/env/config alanlarının sabitlenmesi,
- tek etap sonunda hangi deliverable'ların “tam” sayılacağının ölçülmesi.

Tek etap tamamlanmadan bir sonraki program sürümüne geçilmez.

---

## 2. Tek etap yaklaşımının nedeni

Çok etaplı modellerde şu riskler oluşabilir:

- docs başka yerde, gerçek altyapı başka yerde kalır,
- testnet işleri tamamlandı sanılır ama mainnet tarafı açık kalır,
- sürüm etiketi vardır ama içerik dağınık kalır,
- bazı klasörler/artefact alanları placeholder olarak unutulur,
- ekip “sonra tamamlarız” mantığıyla kritik boşlukları taşır.

Bu belge bu riski reddeder.

Bu programda yöntem şudur:

- önce tüm alanlar tek listede yazılır,
- sonra hepsi aynı etap içinde sahiplenilir,
- hiçbir alan “ileride bakarız” diye açık bırakılmaz,
- program ancak tüm kapanış kriterleri geçtiğinde tamamlanmış sayılır.

---

## 3. Tek etap ana hedefi

### Birincil hedef
AOXChain için **testnet ve mainnet hizalı, belgeye dayalı, sürüme bağlanmış, operasyona hazır, güvenlik kapıları tanımlı ve altyapı eksikleri kapatılmış** bir temel oluşturmak.

### Sonuç cümlesi
Tek etap bittiğinde aşağıdaki soru için cevap **evet** olmalıdır:

> “Repo içinde ne var, nerede var, neden var, nasıl çalıştırılır, hangi sürümde geçerlidir ve testnet ile mainnet için ne seviyede hazırdır?”

---

## 4. Tek etap kapsamı

Bu tek etap aşağıdaki alanların tamamını birlikte kapatır:

1. docs bütünlüğü,
2. mdBook bütünlüğü,
3. sürümleme ve release başlangıç notları,
4. eksik altyapı dosyaları ve klasör yerleşimi,
5. config / fixture / artifact dizinlerinin açıklığı,
6. node-run / servis akışı tanımı,
7. P2P / ağ doğrulama planı,
8. consensus güvenlik çekirdeği gereksinimleri,
9. recovery / snapshot / restore / rejoin çerçevesi,
10. RPC ve public yüzey güvenliği,
11. observability / telemetry / soak beklentileri,
12. release / provenance / rollback / upgrade planı,
13. launch gate ve go/no-go yönetimi,
14. owner / sorumluluk matrisi,
15. kanıt paketleri ve artefact disiplini.

---

## 5. Tek etap çıktı modeli

Tek etap sonunda yalnızca belge üretilmiş olması yeterli değildir. Aşağıdaki bütün çıktılar birlikte tamamlanmalıdır.

### 5.1 Dokümantasyon çıktıları
- ana program belgesi tamamlanmış olmalı,
- `SUMMARY.md` gezinmesi doğru olmalı,
- ilişkili belgeler arasında kırık referans olmamalı,
- testnet, mainnet, network validation, recovery, security ve readiness belgeleri birbirine bağlanmalı,
- her önemli klasör veya dosya ailesi için “ne işe yarar” açıklaması bulunmalı.

### 5.2 Yapısal altyapı çıktıları
- eksik klasör yerleri tanımlanmalı,
- fixture/config/artifact/output dizinleri standardize edilmeli,
- placeholder içerikler ayıklanmalı veya backlog'a bağlanmalı,
- “dosya var ama amacı belirsiz” durumu kalmamalı,
- örnek config ve örnek çalışma yolları tam yazılmalı.

### 5.3 Operasyon çıktıları
- node başlatma, durdurma, doğrulama ve sağlık kontrol akışı yazılmalı,
- operatör hangi komutu hangi sırayla çalıştıracağını net olarak görmeli,
- recovery ve incident süreçleri belgeler arasında tutarlı olmalı,
- launch öncesi kim neyi onaylayacak açıkça yazılmalı.

### 5.4 Güvenlik ve release çıktıları
- sürüm etiketiyle ilişkili release başlangıç notu olmalı,
- rollback / migration / upgrade yaklaşımı yazılı olmalı,
- security owner ve release owner net olmalı,
- testnet için kabul edilen riskler ile mainnet için kabul edilmeyen riskler ayrılmalı.

---

## 6. Tamamlama iş paketleri

## 6.1 Paket A — Docs ve mdBook %100 kapanış

### Hedef
Belge tarafında hiçbir belirsizlik bırakmamak.

### Yapılacaklar
- `docs/src/` içindeki temel belgeleri tek tek gözden geçir,
- eksik başlık, eksik içerik, eksik bağlam ve kırık akışları kapat,
- `SUMMARY.md` içinde görünmesi gereken tüm ana belgeleri ekle,
- başlık adlarını anlamlı ve tutarlı hale getir,
- hangi belge strateji, hangisi runbook, hangisi checklist, hangisi teknik plan açıkça ayrıştır.

### Tamamlanmış sayılma şartı
- yeni katkı yapan biri docs ağacını okuyunca yolunu kaybetmez,
- menüde görünen yapı ile repo gerçek yapısı çelişmez,
- docs tarafında bilinen açık boşluk kalmaz.

## 6.2 Paket B — Sürümleme ve `v0.1.1-alpha` başlangıç bazı

### Hedef
Programı versiyonsuz bırakmamak.

### Yapılacaklar
- `v0.1.1-alpha` etiketinin kapsamını yaz,
- bu sürümün neden başlangıç bazı olduğunu açıkla,
- bu sürüm altında kapanacak alanları sabitle,
- bir üst sürüme geçiş şartlarını yaz.

### Tamamlanmış sayılma şartı
- herkes `v0.1.1-alpha`'nın neyi temsil ettiğini bilir,
- sürüm adı ile içerik arasında boşluk kalmaz.

## 6.3 Paket C — Altyapı dosyaları ve klasör yerleşimi

### Hedef
“Eksik dosya”, “bu klasör ne için”, “artifact nereye düşecek” gibi soruları sıfırlamak.

### Yapılacaklar
- config dizinlerini listele,
- fixture dizinlerini listele,
- artifact çıkış dizinlerini tanımla,
- validation ve soak çıktılarının yerini yaz,
- eksik dosyalar varsa ya üret ya da neden eksik olduğunu resmî backlog maddesine çevir.

### Tamamlanmış sayılma şartı
- repo içindeki kritik dosya aileleri belgelenmiş olur,
- ops, qa ve engineering aynı dizin sözlüğünü kullanır.

## 6.4 Paket D — Node, network ve servis çalıştırma akışı

### Hedef
Çalıştırma yolu tek ve anlaşılır olsun.

### Yapılacaklar
- node bootstrap akışını yaz,
- node-run / servis modu beklentisini netleştir,
- health/readiness kontrolünün nerede yapıldığını yaz,
- gerçek ağ testi ile local smoke farkını ayır.

### Tamamlanmış sayılma şartı
- operatör “hangi komut demo, hangi komut servis, hangi komut validation” ayrımını net görür.

## 6.5 Paket E — Consensus, recovery ve güvenlik kapanış matrisi

### Hedef
Testnet ve mainnet arasında kritik güvenlik boşluğu bırakmamak.

### Yapılacaklar
- consensus çekirdeği için zorunlu gereksinimleri tek listede topla,
- recovery/snapshot/rejoin gereksinimlerini bağla,
- RPC security ve public surface beklentilerini yaz,
- testnet için kabul edilebilen ile mainnet için blocker olan riskleri ayır.

### Tamamlanmış sayılma şartı
- teknik ekip “hangi eksik testnet blocker, hangisi mainnet blocker” sorusunu tek sayfadan cevaplayabilir.

## 6.6 Paket F — Launch gate ve karar yönetimi

### Hedef
Program bitince karar mekanizması net olsun.

### Yapılacaklar
- go/no-go çıktısını tanımla,
- owner listesini yaz,
- launch review toplantısında hangi belgelerin zorunlu olduğunu belirt,
- residual risk kaydını formatla.

### Tamamlanmış sayılma şartı
- launch kararı sezgisel değil, belgeye dayalı hale gelir.

---

## 7. Sahiplik matrisi

Tek etap ancak sahiplik açıksa tamamlanabilir.

### Zorunlu roller
- **Docs Owner:** belge bütünlüğü ve mdBook doğruluğu
- **Release Owner:** sürüm etiketi, release scope, promotion kararı
- **Infra Owner:** klasör yapısı, artifact yerleri, çalışma ortamı beklentileri
- **Network Owner:** gerçek ağ doğrulama, peer/topology beklentileri
- **Consensus Owner:** güvenlik çekirdeği, finality ve state kuralları
- **Security Owner:** RPC/public surface/replay/auth varsayımları
- **Ops Owner:** runbook, incident, recovery, launch operasyonu

### Kural
Bu roller isimlendirilmeden tek etap kapatılamaz.

---

## 8. Zorunlu checklist

Aşağıdaki listenin tamamı **evet** olmadan tek etap kapanmaz.

### 8.1 Docs / md checklist
- [ ] Ana belge tam ve tutarlı
- [ ] `SUMMARY.md` tam
- [ ] mdBook yapısı kırık değil
- [ ] İlişkili belge bağlantıları anlamlı
- [ ] Strateji / runbook / checklist ayrımı net

### 8.2 Version / release checklist
- [ ] `v0.1.1-alpha` kapsamı yazıldı
- [ ] Başlangıç release notu yazıldı
- [ ] Bir üst sürüme geçiş şartı tanımlandı
- [ ] Sürüm adı ile kapsam uyumlu

### 8.3 Infra checklist
- [ ] Kritik config dizinleri yazıldı
- [ ] Fixture yerleri yazıldı
- [ ] Artifact/output yerleri yazıldı
- [ ] Eksik dosyalar üretildi veya backlog'a bağlandı
- [ ] Dizin sözlüğü tamamlandı

### 8.4 Runtime / network checklist
- [ ] Node başlatma akışı açık
- [ ] Servis modu beklentisi açık
- [ ] Health/readiness açıklığı var
- [ ] Local smoke ile real network farkı yazıldı
- [ ] Validation çıktılarının yeri belli

### 8.5 Security / recovery checklist
- [ ] Consensus kapanış maddeleri tek listede
- [ ] Recovery/snapshot çerçevesi bağlı
- [ ] RPC security beklentileri yazıldı
- [ ] Testnet/Mainnet risk ayrımı yazıldı
- [ ] Launch gate blocker'ları net

### 8.6 Governance checklist
- [ ] Owner'lar atandı
- [ ] Go/No-Go formatı yazıldı
- [ ] Residual risk formatı yazıldı
- [ ] Launch review girdileri yazıldı

---

## 9. Zorunlu artefact paketi

Tek etap kapanırken şu paketler bulunmalıdır:

1. program belgesi,
2. güncel `SUMMARY.md`,
3. sürüm kapsam notu,
4. owner matrisi,
5. infra dizin sözlüğü,
6. config/fixture/artifact yer listesi,
7. launch review girdi listesi,
8. residual risk kaydı,
9. testnet/mainnet readiness bağlama notu.

---

## 10. Go / No-Go kuralı

Aşağıdaki durumlardan biri varsa sonuç **No-Go** olur:

- docs tarafında kritik boşluk varsa,
- mdBook gezintisinde belirsizlik varsa,
- `v0.1.1-alpha` kapsamı yazılmadıysa,
- eksik altyapı dosyaları/yerleri tanımlanmadıysa,
- owner listesi boşsa,
- launch gate için gerekli girişler eksikse,
- testnet ve mainnet readiness bağlamı ayrıştırılmadıysa.

Aşağıdaki durumların tamamı sağlanırsa sonuç **Go** olabilir:

- belge tarafı tam,
- sürüm tarafı tam,
- altyapı haritası tam,
- operasyon akışı tam,
- risk formatı tam,
- sahiplik tam,
- launch review girdileri tam.

---

## 11. Başarı tanımı

Bu tek etap programı şu durumda başarılı kabul edilir:

- AOXChain deposu belge ve altyapı açısından dağınık görünmez,
- `v0.1.1-alpha` başlangıç bazı netleşmiş olur,
- testnet ile mainnet aynı program altında konuşulabilir hale gelir,
- eksik dosya/yer/sürüm/owner soruları cevaplanmış olur,
- sonraki teknik uygulama işleri artık belirsizlikle değil, net bir altyapı tabanı üstünde ilerler.

---

## 12. Uygulama talimatı

Bu belgeyi şu şekilde kullan:

1. Önce bu tek etap checklist'ini doldur.
2. Eksik dosya ve klasörleri yaz.
3. `v0.1.1-alpha` kapsamını sabitle.
4. Owner'ları ata.
5. Launch review girdilerini tanımla.
6. Ancak bundan sonra yeni teknik geliştirme etaplarına geç.

Yani karar nettir:

> Önce tek etapta altyapı ve belge tabanı %100 kapanacak, sonra ileri uygulama işleri başlayacak.
