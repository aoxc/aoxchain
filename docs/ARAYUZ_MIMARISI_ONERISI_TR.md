# AOXChain Arayüz (UI/UX) Mimarisi Önerisi (Detaylı v2)

Bu doküman, AOXChain için tasarlanacak web arayüzünün (dashboard + explorer + operatör paneli + köprü görünümü) **ürün seviyesi bilgi mimarisini**, menü yapısını ve ekran içeriklerini önerir.

Bu sürümde özellikle:
- Menülerin detay seviyesi artırılmış,
- Kullanıcı akışları netleştirilmiş,
- **XLayer üzerinde bulunan AOXC ile AOXChain üzerindeki AOXC’nin arayüzde nasıl gösterileceği** ayrıntılandırılmıştır.

---

## 1) Ürün hedefi ve tasarım ilkesi

Arayüzün temel hedefi: farklı profillerin (topluluk, trader, validator, geliştirici, güvenlik ekibi) aynı platformda ama kendi ihtiyaçlarına göre sadeleştirilmiş bir deneyim alması.

### Ana ilke seti
1. **Önce güven, sonra hız:** transfer ve köprü işlemlerinde hata önleyici doğrulamalar.
2. **Önce özet, sonra detay:** her modül girişinde KPI kartları, altta derin analiz.
3. **Aksiyon odaklılık:** metrik varsa yanında “ne yapmalıyım?” önerisi.
4. **Ağlar arası netlik:** AOXC bakiyesinde ağ ayrımı asla belirsiz bırakılmamalı.
5. **Rol bazlı sadeleşme:** herkes için aynı menü değil, aynı veri modeli + farklı görünüm.

---

## 2) Kullanıcı personalları ve öncelikleri

1. **Topluluk Kullanıcısı / Yatırımcı**
   - Cüzdan, toplam AOXC, staking getirisi, governance oyu.
2. **Aktif İşlem Yapan Kullanıcı (Trader/Power User)**
   - Ağ bazlı bakiye, köprü maliyeti/süresi, işlem geçmişi, explorer.
3. **Node Operatörü / Validator**
   - Uptime, peer durumu, slashing riski, performans trendi.
4. **Geliştirici**
   - RPC/API, testnet-mainnet endpoint, sözleşme ve event izleme.
5. **Güvenlik/Uyum Ekibi**
   - Risk skoru, alarm, audit trail, olay müdahale metrikleri.

---

## 3) Üst seviye menü yapısı (öneri)

1. **Dashboard**
2. **Cüzdan**
3. **Transfer & Köprü (Bridge)**
4. **Staking & Validatorlar**
5. **Governance**
6. **Explorer**
7. **Ağ Durumu (Network Ops)**
8. **Güvenlik & Risk**
9. **Geliştirici Merkezi**
10. **Ayarlar & Profil**

> Önceki taslağa göre “Transfer & Köprü” üst menüye çıkarıldı. Sebep: kullanıcıların en kritik hata yaptığı alan ağ-karıştırma ve köprü işlemleri.

---

## 4) Dashboard (ana ekran)

### 4.1 Üst KPI şeridi
- AOXChain blok yüksekliği
- Finality süresi
- Anlık TPS / 24s ortalama TPS
- Aktif validator sayısı
- Son 24s hata oranı
- Ağ sağlık skoru (0-100)

### 4.2 Kullanıcı varlık özeti
- **Toplam AOXC (Birleşik):** AOXChain AOXC + XLayer AOXC
- Ağ bazlı dağılım yüzdesi (örn. %70 AOXChain / %30 XLayer)
- Kullanılabilir bakiye vs kilitli bakiye (staking/vesting)

### 4.3 Hızlı aksiyonlar
- AOXC gönder
- AOXC köprüle (AOXChain ↔ XLayer)
- Stake et
- Governance oy ver

### 4.4 Canlı akış
- Son bloklar
- Son yüksek hacimli işlemler
- Kritik uyarılar (node down, gecikme artışı, köprü kuyruk yoğunluğu)

---

## 5) Cüzdan modülü (Wallet)

## 5.1 Varlıklar sekmesi
- Token listesi (AOXC varsayılan üstte sabit)
- AOXC kartı tıklanınca:
  - Birleşik bakiye
  - Ağ bazlı alt kırılım
  - Son 7/30 gün bakiye değişimi

## 5.2 AOXC varlık kartı (özel tasarım)

AOXC kartında iki satırlı gösterim önerisi:

- **AOXC (Toplam):** 12,450.25 AOXC
- Alt satır:
  - **AOXChain:** 8,730.10 AOXC
  - **XLayer:** 3,720.15 AOXC

Kart içinde 3 buton:
1. **Gönder** (mevcut ağ içinde transfer)
2. **Köprüle** (ağlar arası taşıma)
3. **Detay** (işlem geçmişi + ağ bazlı grafik)

### Neden bu gösterim?
- Kullanıcı tek sayı görmek ister (toplam servet algısı),
- Ama yanlış ağdan gönderim hatasını önlemek için alt kırılım zorunludur.

## 5.3 İşlem geçmişi
- Filtreler:
  - Ağ: AOXChain / XLayer / Tümü
  - Tür: Transfer / Bridge In / Bridge Out / Stake / Reward
  - Durum: Pending / Success / Failed
- Satır içi etiket örnekleri:
  - `Bridge Out (AOXChain→XLayer)`
  - `Bridge In (XLayer→AOXChain)`

---

## 6) Transfer & Köprü modülü (kritik)

## 6.1 Transfer ekranı

Alanlar:
- Ağ seçimi (zorunlu)
- Varlık (AOXC)
- Alıcı adres
- Miktar
- Ücret seviyesi

Doğrulama kuralları:
- Yanlış ağ adres formatı uyarısı
- Miktar > kullanılabilir bakiye engeli
- Son adımda “Gönderdiğiniz ağ: XLayer” gibi büyük onay metni

## 6.2 Köprü ekranı (AOXChain ↔ XLayer)

Zorunlu öğeler:
- **From Network:** AOXChain veya XLayer
- **To Network:** karşı ağ
- **Asset:** AOXC
- **Miktar**
- Tahmini köprü süresi
- Toplam maliyet (kaynak ağ gas + köprü ücreti)

Durum adımları (progress bar):
1. Transaction submitted
2. Source chain confirmed
3. Bridge relayer processing
4. Destination mint/release
5. Completed

Hata senaryoları:
- Relayer gecikmesi
- Destination chain congestion
- Timeout sonrası manuel “retry/claim” aksiyonu

## 6.3 Köprü geçmişi
- Her köprü işleminde tek satır yerine birleşik görünüm:
  - Başlangıç tx hash
  - Hedef tx hash
  - Süre
  - Durum
  - Maliyet

---

## 7) Staking & Validatorlar

### 7.1 Validator listesi
- Uptime
- Komisyon
- Toplam delegasyon
- Son 30 gün performans
- Slashing kaydı
- Risk rozeti (Düşük/Orta/Yüksek)

### 7.2 Delegasyon işlemleri
- Stake et
- Unstake
- Redelegate
- Unbonding takip ekranı

### 7.3 Getiri analizi
- Gerçekleşen ödül
- Tahmini APY
- Senaryo simülasyonu (miktar/validator değişimi)

---

## 8) Governance modülü

### 8.1 Aktif teklifler
- Kalan süre
- Katılım oranı
- Geçme eşiği ilerleme çubuğu

### 8.2 Teklif detayı
- Kısa özet (teknik olmayan kullanıcı için)
- Teknik detay (geliştirici/operatör için)
- Etki alanı: ücret / performans / güvenlik

### 8.3 Oy ekranı
- Oy seçenekleri: Yes / No / Abstain / Veto
- Oy sonrası anlık durum güncellemesi

---

## 9) Explorer modülü

### 9.1 Akıllı arama
Tek kutuda:
- blok numarası,
- tx hash,
- adres,
- validator ID algılama.

### 9.2 Çoklu ağ sekmesi
Explorer üstünde ağ sekmesi:
- AOXChain
- XLayer

Arama sonucu kartında mutlaka “Network badge” gösterilmeli.

### 9.3 İşlem detay ekranı
- Tx status
- From/To
- Fee
- Event log
- Eğer köprü işlemiyse çapraz link:
  - “View on destination network”

---

## 10) Ağ Durumu (Network Ops)

### 10.1 Genel sağlık
- Peer sayısı
- Bölgesel dağılım
- Gecikme haritası

### 10.2 Blok üretim kalitesi
- Ortalama blok süresi
- Boş blok oranı
- Reorg olayları

### 10.3 Senkronizasyon
- Node sync seviyeleri
- Snapshot metrikleri

---

## 11) Güvenlik & Risk

### 11.1 Risk panosu
- Validator risk skorları
- Anomali tespiti
- Zincirleme alarm korelasyonu

### 11.2 Alarm merkezi
- Severity filtreleri
- Olay aksiyon önerisi
- Playbook bağlantıları

### 11.3 Audit trail
- Kim / ne zaman / ne yaptı
- Kritik ayar değişikliği geçmişi

---

## 12) Geliştirici Merkezi

### 12.1 API/RPC playground
- Canlı örnek istek/cevap
- Ağ seçimi: AOXChain / XLayer

### 12.2 SDK örnekleri
- JS/TS
- Python
- Rust

### 12.3 Entegrasyon rehberi
- Cüzdanda AOXC çoklu ağ gösterimi için örnek veri modeli
- Köprü durum poll akışı

---

## 13) AOXC’nin AOXChain ve XLayer’da arayüzde gösterimi (detaylı tasarım)

Bu bölüm kullanıcı talebi doğrultusunda özellikle detaylandırılmıştır.

## 13.1 Terminoloji standardı
- **AOXC (Native - AOXChain):** AOXChain üzerindeki AOXC
- **AOXC (Bridged - XLayer):** XLayer üzerinde köprülenmiş/temsil edilen AOXC

Arayüzde sadece “AOXC” yazıp bırakmak yasak olmalı; her yerde ağ etiketi zorunlu.

## 13.2 Varlık gösterim standardı

Her AOXC görünümünde aşağıdaki üç katman olmalı:

1. **Toplam AOXC** (birleşik bilgi)
2. **Ağ bazlı bakiye kırılımı**
3. **Likidite durumu** (kullanılabilir, kilitli, köprüde)

Örnek:
- Toplam: 10,000 AOXC
- AOXChain: 6,000 AOXC (Available 5,500 / Staked 500)
- XLayer: 4,000 AOXC (Available 3,900 / In-Bridge 100)

## 13.3 Renk/rozet sistemi
- AOXChain rozet rengi: mavi (örnek)
- XLayer rozet rengi: mor (örnek)
- Bridge durum rozeti:
  - Pending (sarı)
  - Completed (yeşil)
  - Failed (kırmızı)

## 13.4 Gönderim ekranında AOXC ağ güvenliği

Kullanıcı “AOXC gönder”e bastığında:
1. Önce ağ seçimi zorunlu,
2. Seçilen ağa göre bakiye gösterimi,
3. Hedef adres formatı/ağ uyumluluğu kontrolü,
4. Son onay ekranında büyük fontla:
   - “XLayer ağında AOXC gönderiyorsunuz” veya
   - “AOXChain ağında AOXC gönderiyorsunuz”.

## 13.5 Köprü akışında AOXC gösterimi

Köprü ekranında aynı anda iki bakiye gösterilmeli:
- Kaynak ağ AOXC bakiyesi,
- Hedef ağ tahmini alınacak AOXC.

Köprü onayı sonrası kullanıcıya işlem kimliğiyle birlikte:
- Kaynak tx hash,
- Hedef tx hash (oluşunca),
- Kalan tahmini süre
sunulmalı.

## 13.6 Explorer entegrasyonu

Köprülenmiş AOXC işlemlerinde işlem detay sayfası:
- “Bu işlem bir bridge işlemidir” banner’ı,
- Kaynak/hedef ağ bağlantıları,
- Köprü mesaj ID’si,
- Durum timeline.

## 13.7 Portföy ekranında yanlış algıyı önleme

Sık hata: kullanıcı XLayer’daki AOXC’yi AOXChain’de kullanabileceğini sanır.

Bunu önlemek için:
- Portföy kartında “Ağlar arası otomatik birleşmez” bilgi notu,
- “Bu bakiyeyi diğer ağda kullanmak için köprü gerekir” CTA,
- Tek tıkla köprü ekranına geçiş.

## 13.8 API veri modeli önerisi (UI tüketimi için)

```json
{
  "symbol": "AOXC",
  "total_balance": "10000",
  "networks": [
    {
      "network": "AOXCHAIN",
      "type": "native",
      "available": "5500",
      "staked": "500",
      "in_bridge": "0"
    },
    {
      "network": "XLAYER",
      "type": "bridged",
      "available": "3900",
      "staked": "0",
      "in_bridge": "100"
    }
  ]
}
```

Bu modelle UI’da tek çağrıda hem toplam hem ağ kırılımı gösterilebilir.

---

## 14) Rol bazlı menü görünümü

### 14.1 Temel kullanıcı modu
- Dashboard
- Cüzdan
- Transfer & Köprü
- Staking
- Governance
- Explorer

### 14.2 Operatör modu
- Dashboard
- Network Ops
- Validatorlar
- Güvenlik & Risk
- Explorer

### 14.3 Geliştirici modu
- Dashboard
- Explorer
- Geliştirici Merkezi
- Network Ops (read-only metrik görünümü)

---

## 15) UX mikro-kopya önerileri (Türkçe)

- “AOXC bakiyeniz iki ağda tutuluyor. İşlem öncesi ağ seçimini kontrol edin.”
- “Bu transfer XLayer üzerinde gerçekleşecek.”
- “AOXChain’de kullanmak için bu varlığı köprülemeniz gerekir.”
- “Köprü işlemi ağ yoğunluğuna bağlı olarak gecikebilir.”

---

## 16) MVP ve Faz-2 planı

## MVP
- Dashboard (temel KPI)
- Cüzdan (AOXC ağ kırılımı dahil)
- Transfer ekranı
- Köprü ekranı (AOXChain ↔ XLayer)
- Explorer (çoklu ağ sekmesi)
- Staking (temel)
- Governance (oy verme)

## Faz-2
- Gelişmiş risk skorları
- Olay müdahale merkezi
- Geliştirici playground
- Gelişmiş portföy analitiği

---

## 17) Başarı metrikleri

- İlk transfer başarı oranı
- Köprü işlem tamamlama oranı
- Yanlış ağ seçimi kaynaklı hata oranı
- AOXC ağ-kırılım ekranı görüntülendikten sonra hata düşüşü
- Governance katılım artışı
- Kullanıcı destek taleplerinde “ağ karışıklığı” başlığında azalma

---

## 18) Sonuç

En doğru yaklaşım, AOXC’yi tek bir etiketle göstermek yerine **toplam + ağ bazlı kırılım + işlem bağlamı** ile sunmaktır. AOXChain ve XLayer’daki AOXC aynı marka varlık deneyiminin parçası gibi görünmeli; ancak teknik gerçeklik (network farkı, köprü gereksinimi, ücret/süre farklılığı) arayüzde açık ve hatasız aktarılmalıdır. Böylece hem kullanıcı hataları azalır hem güven artar.
