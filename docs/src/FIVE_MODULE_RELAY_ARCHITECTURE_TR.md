# AOXChain Beş Modüllü Relay Mimari Önerisi

Bu doküman, AOXChain için önerilen **ince relay core + 5 bağlı fonksiyonel modül** mimarisini tanımlar.

## 1. Ana ilke

Relay chain mümkün olduğunca:

- ince,
- tarafsız,
- kalıcı,
- ve düşük saldırı yüzeyli

olmalıdır.

Relay core'un işi uygulama mantığını taşımak değil; güvenlik, sıralama, yönetişim ve kanonik state commitment omurgası olmaktır.

## 2. Relay core ne yapmalı?

Relay core yalnızca aşağıdaki anayasal görevleri üstlenmelidir:

1. finality / ordering
2. shared security
3. validator set management
4. cross-module message routing
5. universal identity root
6. state commitment / proof-root anchoring
7. governance and upgrades
8. fee and staking settlement root

## 3. Neden 5 modül?

Zincir ailelerine göre parçalamak yerine işlevlere göre parçalamak daha stabildir. EVM, Solana, UTXO, IBC veya object-centric aileler zamanla değişebilir; fakat kimlik, varlık, execution, interop ve proof sorumlulukları kalıcıdır.

Bu yüzden AOXChain için önerilen fonksiyonel modüller:

1. `AOXC-MODULE-IDENTITY`
2. `AOXC-MODULE-ASSET`
3. `AOXC-MODULE-EXECUTION`
4. `AOXC-MODULE-INTEROP`
5. `AOXC-MODULE-PROOF`

## 4. Modül tanımları

### 4.1 Identity Module

Amaç:

- `aoxc1...` universal address / identity handle
- chain-specific address binding
- reverse lookup
- recovery rules
- key rotation
- delegates / permissions

Tutulan temel kayıtlar:

- AOXC Universal ID
- bağlı EVM adresleri
- bağlı Solana adresleri
- bağlı diğer zincir hesapları
- guardian / recovery policy hash
- metadata hash

### 4.2 Asset & Treasury Module

Amaç:

- native AOXC asset mantığı
- wrapped asset kayıtları
- bridge escrow kayıtları
- treasury accounting
- settlement balances
- fee accounting

Bu ayrım, bridge veya wrapped-asset riskinin tüm sistemi etkilemesini önler.

### 4.3 Smart Execution Module

Amaç:

- contract execution
- programmable actions
- intent settlement
- app-specific logic

Bu katman relay core'un dışında tutulmalıdır; çünkü en hızlı değişen, en çok upgrade isteyen ve en yüksek saldırı yüzeyine sahip alan burasıdır.

### 4.4 Interop / Bridge Module

Amaç:

- dış zincir bağlantıları için tek domain
- inbound / outbound nonce tracking
- replay protection
- proof verification records
- adapter-based interoperability

Önerilen adapter aileleri:

- EVM Adapter
- Solana Adapter
- UTXO Adapter
- IBC Adapter
- Object Adapter

Kural:

> Her zincir için ayrı bridge ürünü değil, tek interop modülü içinde adapter ailesi yaklaşımı.

### 4.5 Data / Proof Module

Amaç:

- data commitments
- blob / batch commitments
- proof publication
- light-client support data
- fraud / validity proof references

Bu katman execution ölçeklemesi ve dış doğrulama için kritik olur.

## 5. Relay ile modül arasındaki zorunlu bağlar

Her modül aşağıdaki dört eksende relay core'a bağlı olmalıdır:

1. shared validator security veya relay-finalized checkpoint acceptance
2. message bus
3. identity root dependency
4. state commitment anchoring

Bu sayede modüller esnek kalırken, son geçerlilik ve anayasal güven relay core'da kalır.

## 6. AOXC Message Envelope önerisi

Hem modüller arası hem de dış zincir mesajları için tek format önerisi:

- `sourceModule`
- `destinationModule`
- `sourceChainFamily`
- `targetChainFamily`
- `nonce`
- `payloadType`
- `payloadHash`
- `proofReference`
- `feeClass`
- `expiry`
- `replayProtectionTag`

## 7. Güvenlik sınırları

### Relay core

- minimum attack surface
- critical state only
- no heavy app logic
- governance-controlled upgrades

### Modüller

- separate risk domains
- separate rate limits
- separate circuit breakers
- separate fee policies
- separate storage proof domains

## 8. AOXChain için pratik sonuç

AOXChain, tüm mantığı tek zincirde toplamaya çalışmak yerine şu yapıya yönelmelidir:

- `AOXC-RELAY-CORE`
- `AOXC-MODULE-IDENTITY`
- `AOXC-MODULE-ASSET`
- `AOXC-MODULE-EXECUTION`
- `AOXC-MODULE-INTEROP`
- `AOXC-MODULE-PROOF`

Bu model:

- relay chain'i aplikasyon zincirine dönüştürmez,
- güvenlik ve yönetişimi merkezde tutar,
- zincir uyumluluğunu adapter ailesi ile çözer,
- ileride yeni zincir aileleri geldiğinde mimariyi bozmaz.
