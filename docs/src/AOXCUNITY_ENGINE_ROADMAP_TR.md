# AOXCUnity Deterministic Consensus Engine Roadmap (TR)

Bu belge, `aoxcunity` için mevcut durum ile hedef durum arasındaki farkı netleştirir.
Amaç, crate'i “iyi ayrılmış consensus primitive koleksiyonu” seviyesinden
“deterministic consensus engine with persistent replay safety and operator-grade
failure semantics” seviyesine taşımaktır.

## 1. Mevcut durumun kısa özeti

`aoxcunity` içinde bugün güçlü bir iskelet vardır:

- `block`, `fork_choice`, `quorum`, `rotation`, `safety`, `seal`, `store`, `vote_pool`
  gibi modüller ayrışmıştır.
- `kernel.rs`, event/certificate/effect/rejection/transition sonucu için ortak
  sözleşmeyi tanımlar.
- `state.rs`, block admission, quorum gözlemi ve finalize denemesi gibi temel
  yardımcı akışları içerir.

Buna rağmen crate şu aşamada henüz “tam çalışan production consensus subsystem”
olarak konumlandırılmamalıdır. En kritik eksik, tek giriş noktalı ve her event
başına tek deterministic state transition üreten açık bir orchestrator motorunun
henüz crate seviyesinde tamamlanmamış olmasıdır.

## 2. Resmî konumlandırma

### 2.1 Bugünkü tanım

`aoxcunity` şu an için en doğru şekilde şöyle tanımlanmalıdır:

> Consensus primitives + constitutional finality domain.

### 2.2 Hedef tanım

Hedef durum şu olmalıdır:

> Deterministic consensus engine with persistent replay safety and
> operator-grade failure semantics.

Bu ayrım özellikle release notları, readiness iddiaları ve operator belgelerinde
korunmalıdır.

## 3. Faz 1 — Şimdi yapılmalı

### 3.1 Tek merkezli consensus orchestrator

İlk büyük hedef, `ConsensusEngine` veya `ConsensusKernel` adlı tek merkezli bir
orchestrator oluşturmaktır.

#### Girdi

- `ConsensusEvent`

#### Merkezi state

- round state
- validator set snapshot
- fork choice
- lock state
- vote pool
- finalized seal / certificate state
- evidence buffer
- recovery bookkeeping

#### Çıktı

- `TransitionResult`

#### Zorunlu kural

- tek event → tek deterministic state transition

Bu orchestrator aşağıdaki zinciri açık biçimde birleştirmelidir:

1. event admission,
2. safety precondition kontrolü,
3. vote / timeout / certificate işleme,
4. fork-choice güncellemesi,
5. quorum değerlendirmesi,
6. finality üretimi,
7. pruning,
8. invariant denetimi.

### 3.2 State modelinin merkezileştirilmesi

Consensus ile ilgili dağınık in-memory state tek bir top-level kernel state
altında toplanmalıdır. Böylece replay, snapshot alma, invariant doğrulama ve test
senaryoları tek merkezden yönetilebilir.

### 3.3 Recovery semantics'in gerçek replay modeline yükseltilmesi

`store.rs` içindeki soyutlamalar yararlı bir başlangıçtır; ancak operational
recovery discipline henüz tamamlanmış sayılmamalıdır.

İlk sertleştirme adımları:

- persisted event format version alanı,
- snapshot versioning,
- snapshot + journal integrity hash,
- snapshot sonrası journal replay boundary,
- replay sırasında invariant verification,
- corrupted journal / torn write recovery politikası,
- “recovered state == live-derived state” property testleri,
- crash consistency senaryoları.

## 4. Faz 2 — Consensus-grade sertleştirme

### 4.1 Safety rules derinleştirilmeli

`safety.rs` bugünkü haliyle iyi bir başlangıçtır; ancak yalnızca regression check
seviyesinde bırakılmamalıdır.

Eklenmesi gerekenler:

- explicit `SafetyRules` dokümanı,
- her vote path için precondition matrisi,
- ancestor-extension requirement formalizasyonu,
- conflicting QC / conflicting justification handling,
- timeout path ile normal vote path için ayrı güvenlik kuralları,
- cross-epoch validator set transition safety,
- equivocation evidence'den türeyen slashing / exclusion hook'ları.

### 4.2 Fork-choice production seviyesine taşınmalı

Mevcut minimal politika erken prototip için yeterlidir; ancak production için şu
sıra önerilir:

1. finalized anchor,
2. justified head preference,
3. highest certified descendant,
4. deterministic tie-break,
5. stale / invalid certified branch rejection,
6. optimistic head vs safe head ayrımı.

### 4.3 Timeout ve certificate akışı netleştirilmeli

Round advancement ve liveness recovery akışı belirsiz kalmamalıdır.
Açık kurallar gereklidir:

- timeout certificate üretim eşiği,
- timeout certificate'in lock advancement ile ilişkisi,
- proposer fallback/advancement kuralları,
- timeout sonrası proposer değişiminin deterministik modeli.

### 4.4 Evidence lifecycle tamamlanmalı

`vote_pool` içindeki equivocation tespiti doğrudan evidence pipeline'a
bağlanmalıdır.

Gerekli bileşenler:

- `EquivocationEvidence` üretimi,
- `EvidenceStore` entegrasyonu,
- slashable offense modeli,
- invalid signature quarantine,
- memory bound ve eviction policy,
- anti-DoS admission sınırları.

## 5. Faz 3 — Mainnet readiness disiplini

`aoxcunity` için iki ayrı seviye açıkça ayrılmalıdır:

- **library complete**: deterministic core modülleri + orchestrator + replay safety
- **network-ready**: node/runtime/network entegrasyonları + operatör prosedürleri

Mainnet-readiness yönünde ayrıca şunlar dokümante edilmelidir:

- consensus-sensitive rollback procedure,
- invariant bozulursa node'un nasıl davrandığı,
- release gate / exception politikası,
- operator-visible failure semantics.

## 6. Yakın dönem release gate önerisi

Aşağıdaki maddeler tamamlanmadan `aoxcunity` için production-grade consensus
çekirdeği iddiası yapılmamalıdır:

- deterministic `apply_event` motoru,
- replayable persistent recovery modeli,
- explicit safety rules + test matrisi,
- justified/certified-aware fork choice,
- timeout/certificate akışı,
- structured equivocation evidence lifecycle,
- validator-set-aware quorum verification.

## 7. Minimal dependency genişletme önerisi

Bağımlılıkların sade tutulması değerlidir; ancak şu alanlarda kontrollü genişleme
uygundur:

- `tracing`,
- deterministic test utilities,
- structured error surface,
- persistence / network adapter'ları için optional feature flags.

Amaç “ağır dependency doldurmak” değil; production-grade işletim ve test
kabiliyetini kontrollü şekilde artırmaktır.
