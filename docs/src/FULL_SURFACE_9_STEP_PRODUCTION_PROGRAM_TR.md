# Full Surface 9 Adım Üretim Programı

Bu belge, AOXChain’i **mainnet + testnet + AOXHub + wallet + telemetry** yüzeylerini birlikte ele alarak üretim seviyesine taşımak için kullanılacak 9 adımlı tam kapanış planıdır.

> Bu turda özellikle **1. adım** başlatıldı: tüm yüzeylerin tek bir readiness matrisi altında toplanması.

## Kapsam

Bu program aşağıdaki yüzeyleri birlikte yönetir:

- **Mainnet**
- **Testnet**
- **AOXHub**
- **Desktop / wallet istemcileri**
- **Telemetry / observability / audit evidence**

Tek bir yüzeyin “hazır” olması yeterli değildir. Üretim iddiası ancak bu yüzeylerin **aynı release hattında tutarlı**, **kanıtlanabilir** ve **geri alınabilir** olmasıyla yapılabilir.

## 9 adımın özeti

### 1. Tekil readiness matrisi ve kanıt haritası

Amaç:

- tüm üretim yüzeylerini aynı tabloda toplamak,
- her yüzey için sahip, kanıt, komut ve blocker tanımlamak,
- “eksik ama hazır görünüyor” riskini ortadan kaldırmak.

Bu adımın repository çıktısı:

- `models/full_surface_readiness_matrix_v1.yaml`

Tamamlanma kriteri:

- mainnet/testnet/hub/wallet/telemetry için en az birer zorunlu kanıt satırı tanımlı olmalı,
- her satırda owner, evidence, command ve blocker alanları bulunmalı,
- release owner tek bakışta hangi yüzeyin bloke olduğunu görebilmeli.

### 2. Build + CI + release gate kapanışı

Amaç:

- format/lint/test/security/build/provenance zincirini tek release kapısında birleştirmek,
- “green test ama eksik release gate” durumunu kaldırmak.

Beklenen çıktı:

- locked build,
- clippy + tests + audit + deny + release evidence akışı,
- kırmızı/yeşil release kararı.

### 3. Mainnet ve testnet profil ayrışması

Amaç:

- port, bind, logging, key policy ve rollout davranışlarını profillere göre kesin ayırmak,
- testnet kolaylıklarının mainnet’e sızmasını engellemek.

Beklenen çıktı:

- profile drift kontrolü,
- net fark listesi,
- promotion öncesi uyumluluk raporu.

### 4. AOXHub ve wallet akış uyumluluğu

Amaç:

- hub API, signing, route ve kullanıcı akışlarını mainnet/testnet ile eşlemek,
- wallet tarafında environment confusion riskini azaltmak.

Beklenen çıktı:

- hub/wallet compatibility matrix,
- signing flow parity kontrolleri,
- route/surface sapma raporu.

### 5. Telemetry + alerting + incident evidence kapanışı

Amaç:

- readiness score, health, RPC, consensus ve security sinyallerini izlenebilir hale getirmek,
- alarmdan audit evidence’a uzanan zinciri somutlaştırmak.

Beklenen çıktı:

- telemetry snapshot,
- alert rules,
- incident sonrası doğrulanabilir artefact seti.

### 6. Multi-host ağ doğrulaması ve recovery

Amaç:

- tek node güvenini kırıp gerçek ağ koşullarını doğrulamak,
- partition, restart, recovery ve sync davranışını kanıtlamak.

Beklenen çıktı:

- multi-host validation raporu,
- recovery evidence,
- soak ve failure senaryosu sonuçları.

### 7. Güvenlik ve anahtar yaşam döngüsü kapanışı

Amaç:

- operator key, revocation, rotation, genesis ve custody yüzeylerini operasyonel olarak sertleştirmek,
- audit trail’i release zinciriyle bağlamak.

Beklenen çıktı:

- key rotation evidence,
- hardened permissions doğrulaması,
- genesis reproducibility kanıtı.

### 8. Operasyon runbook ve rollback pratiği

Amaç:

- on-call, incident, rollback, restart, degraded mode ve recovery akışlarını çalışan prosedüre dönüştürmek.

Beklenen çıktı:

- güncel runbook seti,
- rollback kriterleri,
- tatbikat kaydı.

### 9. Final production closure ve release sign-off

Amaç:

- tüm yüzeylerden gelen kanıtları tek release paketinde toplamak,
- artık “hazırlık” değil “sign-off” seviyesine geçmek.

Beklenen çıktı:

- release evidence paketi,
- readiness yüzdesi + blocker özeti,
- release owner / security / operations imza alanı.

## Bu turda başlatılan 1. adım

İlk adım için prensip şudur:

1. yüzeyleri tek listede toplamak,
2. her yüzey için zorunlu kanıtı tanımlamak,
3. hangi komut veya artefact’in bu kanıtı ürettiğini sabitlemek,
4. eksik durumda blocker’ı açıkça yazmak.

Bu yüzden `models/full_surface_readiness_matrix_v1.yaml` dosyası canonical başlangıç noktası olarak eklendi.

## 1. adımın kullanım şekli

Release veya readiness toplantısında şu sıra izlenmelidir:

1. matrix dosyasını aç,
2. `status != ready` olan satırları filtrele,
3. blocker alanlarını sprint işlerine dönüştür,
4. evidence path veya command alanı boşsa release adayını yükseltme,
5. tüm yüzeylerde owner atanmadıysa sign-off verme.

## Başarı ölçütü

1. adım başarılı sayılırsa:

- tüm full-surface kapsamı görünür olur,
- dağınık readiness belgeleri ortak bir omurgaya bağlanır,
- sonraki 8 adım ölçülebilir hale gelir.
