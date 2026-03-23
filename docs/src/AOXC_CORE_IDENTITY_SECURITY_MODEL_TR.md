# AOXC Core Identity Security Model (TR)

Bu belge, AOXChain node kimliği, sertifika, transport admission, validator yetkisi ve recovery akışlarının **uygulama katmanında değil, mümkün olduğunca çekirdek/core düzeyinde** ele alınması gerektiğini savunur.

Kısa cevap:

> **Evet, bu bölüm core düzeyinde full yapılmalı.**
>
> Uygulama katmanı yalnızca policy tüketicisi olmalı; güvenlik kararı veren son merci ise `aoxcore` + consensus kernel olmalı.

---

## 1. Neden uygulama değil, core?

Eğer kimlik, sertifika, revocation, handshake admission ve validator yetki kontrolleri üst katmanlarda kalırsa şu riskler oluşur:

- farklı servisler farklı doğrulama yapar,
- bir yerde certificate süresi kontrol edilirken başka yerde atlanır,
- app-level bypass ile “geçici” istisnalar kalıcı güvenlik açığına dönüşür,
- consensus ve network aynı node kimliğini farklı yorumlayabilir,
- recovery / rotation mantığı operasyon araçlarına dağılır.

Buna karşılık core-first modelde:

- tek canonical validation surface oluşur,
- bütün node'lar aynı güvenlik kurallarını uygular,
- audit daha kolay yapılır,
- panic yerine deterministik hata akışı korunur,
- güvenlik politikası uygulama mantığından ayrılır.

---

## 2. Çekirdek seviyesinde zorunlu olması gereken güvenlik alanları

Aşağıdaki alanlar uygulama/plugin/CLI seviyesinde bırakılmamalı; **core primitive** olarak tanımlanmalıdır.

### 2.1 Node identity bundle doğrulaması

`NodeIdentityBundleV1` benzeri canonical yapı geldiğinde şu kontroller core içinde olmalı:

- alan bütünlüğü,
- canonical serialization,
- crypto profile uyumluluğu,
- role/public-key eşleşmesi,
- zaman geçerliliği,
- certificate chain tutarlılığı,
- rotation counter monotonicity.

### 2.2 Certificate issuance ve verification

Certificate mantığı string alanlar veya gevşek JSON tüketimi şeklinde kalmamalı.

Core şu garantileri vermeli:

- subject identity canonicalized,
- issuer bağlamı sabit,
- signing domain ayrılmış,
- signature suite açık,
- validity window enforced,
- revoked subject reddedilir,
- issuer mismatch deterministik hata döndürür.

### 2.3 Handshake admission

P2P admission kararı network katmanında “best effort” olmamalı.

Core-level handshake gate şunları enforce etmeli:

- sertifika imzası,
- issuer doğruluğu,
- revocation list kontrolü,
- zaman geçerliliği,
- transport key binding,
- identity bundle ile certificate subject eşleşmesi,
- zone / profile policy uyumu.

### 2.4 Consensus signer admission

En kritik konu budur: her network'e bağlanan node validator değildir.

Bu yüzden core consensus tarafı ayrıca şunları istemeli:

- validator set membership doğrulaması,
- active/eligible kontrolü,
- vote key role doğrulaması,
- rotation sonrası yetki zinciri kontrolü,
- certificate veya attestation ile signer authorization.

### 2.5 Revocation ve key rotation

Revocation operasyonel not olarak değil, **çekirdek state transition** kuralı olarak ele alınmalı.

Core şunları sağlamalı:

- revoked kimlikler yeni handshake alamaz,
- revoked signer consensus oyu veremez,
- rotation kaydı olmadan yeni key kabul edilmez,
- recovery path audit izi olmadan çalışmaz,
- eski ve yeni key arasında canonical continuity kanıtı aranır.

---

## 3. Önerilen çekirdek mimari ayrımı

Benim önerim identity/security tarafını 4 ana seviyeye ayırmak.

### 3.1 Layer 0 — Cryptographic primitives

Burası en alt katmandır:

- hash,
- canonical encoding,
- signature suites,
- key serialization,
- domain-separated signing payload'ları.

Bu katman saf, deterministik ve mümkün olduğunca küçük tutulmalı.

### 3.2 Layer 1 — Identity core model

Bu katman şu typed yapıları içerir:

- `NodeKeyRole`
- `CryptoProfile`
- `HybridPublicKeySet`
- `NodeIdentityBundleV1`
- `NodeCertificateV1`
- `NodeRevocationRecord`
- `NodeRotationRecord`

Bu katmanda kural şudur:

> identity ile ilgili hiçbir kritik karar string parsing veya gevşek JSON alanına bırakılmaz.

### 3.3 Layer 2 — Admission and authorization kernel

Bu katman karar verir:

- peer bağlanabilir mi?
- certificate geçerli mi?
- bu signer vote atabilir mi?
- bu node üretici olabilir mi?
- bu rotation kabul edilmeli mi?

Yani güvenin gerçek enforcement noktası burasıdır.

### 3.4 Layer 3 — Application / operator surface

CLI, bootstrap tool, dashboard, RPC ve automation katmanları sadece core kurallarını çağırmalı.

Bu katman:

- artifact üretir,
- operator'a görünürlük sunar,
- tanı/teşhis sağlar,
- ama güvenlik kuralını değiştiremez.

---

## 4. "Farklı" ve daha güvenli olmak için neyi klasik zincirlerden farklı yapabiliriz?

Sadece "sertifika eklemek" yeterli değil. Gerçek fark için aşağıdaki yaklaşımlar önerilir.

### 4.1 Tek key yerine görev ayrılmış kimlik

Klasik yaklaşım:

- tek validator key,
- aynı key ile network + consensus + operator işleri.

Önerilen AOXC yaklaşımı:

- `NodeIdentityKey`
- `ConsensusVoteKey`
- `TransportKey`
- `OperatorKey`
- `RecoveryKey`
- `PqAttestationKey`

Bu, tek anahtar sızıntısının tüm node güvenliğini çökertmesini engeller.

### 4.2 Hybrid trust stack

Kısa vadede yalnızca klasik imza kullanmak pratik olabilir, ama uzun vadede yeterli olmayabilir.

Daha farklı ve güçlü model:

- classical + PQ hybrid public key set,
- certificate tarafında suite açıkça yazılır,
- profile bazlı kabul politikası uygulanır,
- ağ isterse `hybrid-required` profile geçebilir.

### 4.3 Admission before execution

Bir node önce bağlanıp sonra doğrulanmamalı.

Daha güvenli model:

- önce kimlik doğrulama,
- sonra transport admission,
- sonra consensus eligibility,
- en sonda execution/replication erişimi.

Yani “önce gir sonra bakarız” yaklaşımı olmamalı.

### 4.4 Security policy as state machine

Birçok sistemde güvenlik policy'si config dosyalarında yaşar. Bu zayıf bir modeldir.

Daha sağlam model:

- revocation,
- rotation,
- validator activation,
- validator suspension,
- certificate rollover,
- emergency recovery

bunların hepsi state transition mantığına bağlanmalı.

### 4.5 Evidence-first security

Sadece doğrulama değil, kanıt da üretilmeli.

Core her kritik kararda audit/evidence üretmeli:

- neden kabul edildi,
- neden reddedildi,
- hangi rule tetiklendi,
- hangi issuer / role / epoch kullanıldı,
- hangi rotation chain kabul edildi.

Bu model incident response ve audit için çok değerlidir.

---

## 5. Core düzeyinde eklenmesi gereken sert güvenlik kuralları

### 5.1 Fail-closed davranış

Kararsızlık durumunda sistem kabul etmemeli.

Örnek:

- profile bilinmiyorsa reject,
- certificate parse edilemiyorsa reject,
- revocation store okunamıyorsa reject-or-degraded-safe mode,
- signer role belirsizse reject.

### 5.2 Domain separation everywhere

Aynı key ile farklı domain'lerde aynı payload sınıfı imzalanmamalı.

Ayrı domain'ler:

- identity bundle signing,
- certificate issuance,
- transport handshake,
- consensus vote,
- timeout vote,
- recovery authorization.

### 5.3 Canonical serialization only

Farklı serializer çıktıları farklı hash/signature üretmemeli.

Bu yüzden core tarafında:

- canonical field order,
- explicit versioning,
- deterministic encoding,
- strict byte payload construction

zorunlu olmalı.

### 5.4 No unsigned authority transitions

Aşağıdaki işlemler imzasız veya gevşek doğrulamayla ilerlememeli:

- validator ekleme,
- validator çıkarma,
- key rotation,
- certificate re-issuance,
- emergency recovery,
- trust profile değişimi.

### 5.5 Replay resistance

Identity ve certificate tarafında replay kontrolü de çekirdekte olmalı:

- nonce / session binding,
- issuance epoch bağlamı,
- rotation counter,
- certificate serial veya equivalent id,
- replay cache / replay proof alanları.

---

## 6. Uygulama katmanı nerede kalmalı?

Uygulama katmanı tamamen değersiz değil; ama rolü sınırlı olmalı.

Uygulama tarafında kalabilecek işler:

- dosya üretimi,
- operator UX,
- dashboard görselleştirme,
- raporlama,
- key import/export workflow,
- monitoring entegrasyonu.

Ama şu kararlar app'te kalmamalı:

- “bu node trusted mı?”
- “bu certificate geçerli mi?”
- “bu signer oy kullanabilir mi?”
- “rotation continuity doğru mu?”
- “bu peer consensus kanalına girebilir mi?”

---

## 7. AOXC için önerdiğim net yol

### Aşama 1 — Identity core types

`aoxcore::identity` altında typed yapıların eklenmesi:

- `NodeKeyRole`
- `CryptoProfile`
- `HybridPublicKeySet`
- `NodeIdentityBundleV1`
- `NodeCertificateV1`
- `RotationPolicy`
- `RevocationRecord`

### Aşama 2 — Core validators

Ayrı validate fonksiyonları değil, policy-aware validator yüzeyi:

- `validate_identity_bundle()`
- `validate_node_certificate()`
- `authorize_transport_peer()`
- `authorize_consensus_signer()`
- `authorize_key_rotation()`

### Aşama 3 — Kernel binding

Consensus kernel şu typed identity yüzeyine bağlanmalı:

- block producer authorization,
- vote signer authorization,
- quorum signer set binding,
- finalized certificate signer integrity.

### Aşama 4 — Network gate hardening

P2P admission, transport profile ve replay savunmaları identity core ile aynı modelden beslensin.

### Aşama 5 — Operator tools as thin wrappers

`aoxcmd` ve `aoxckit` sadece bu kuralları kullanan ince araçlar olsun.

---

## 8. Benim net önerim

Eğer amaç:

- daha güvenli,
- daha kaliteli,
- kernel/core düzeyinde çok sağlam,
- uygulama seviyesinde de güven oranı çok yüksek

bir yapı kurmaksa, o zaman **identity + certificate + revocation + admission + rotation** tarafı mümkün olduğunca `aoxcore` içinde kapanmalı.

Yani:

> **Evet, bu bölümü core düzeyinde full yapmak doğru olur.**

Ama bunu yalnızca “daha fazla kodu core'a taşıyalım” diye değil; şu prensiple yapalım:

> **Uygulama katmanı güvenlik policy'sini tanımlamaz; yalnızca çekirdeğin tanımladığı güvenlik kararlarını kullanır.**

Bu ayrım kurulursa AOXC, klasik tek-key validator zincirlerinden daha kontrollü, daha audit-friendly ve daha upgrade-edilebilir bir güvenlik mimarisine sahip olabilir.
