# AOXC Kernel Hardening Master Plan (TR)

Bu belge, `aoxcunity` + `aoxcai` + çevresindeki çekirdek workspaceler için **consensus safety öncelikli**, **AI'nın asla authority olmadığı**, **audit-ready** ve **production-grade** bir AOXC çekirdeğine giden yolu tanımlar.

Belgenin amacı “küçük iyileştirme listesi” sunmak değil; mevcut scaffold yapısını, protokol seviyesinde katılaştırılmış bir kernel mimarisine dönüştürmek için doğrudan uygulanabilir bir ana plan vermektir.

## 1. Mimari hüküm: `aoxcai` asla consensus authority değildir

AOXC'nin değişmez anayasal ilkesi şudur:

> AI assistance may influence operator understanding, but it must never influence consensus truth, validator authority, state transition validity, fork choice, quorum attainment, or finality.

Bu ilke, yalnızca belge düzeyinde değil, kod ve test düzeyinde de zorunlu invariant olarak korunmalıdır.

### 1.1 İzin verilen kullanım alanları

`aoxcai` sadece operator-plane ve advisory alanlarında kalmalıdır:

- diagnostics explanation,
- incident summary,
- remediation draft,
- config review,
- runbook preparation,
- compatibility review,
- operator diagnostics.

### 1.2 Yasak alanlar

`aoxcai` şu alanlara **hiçbir şekilde** bağlanmamalıdır:

- proposer seçimi,
- vote admission,
- quorum hesabı,
- fork choice,
- validator activation/revocation,
- finality kararı,
- slashing/dispute authority,
- state transition validity,
- genesis authority calculation.

### 1.3 Uygulama talimatı

- `aoxcunity`, `aoxcore`, `aoxcnet`, `aoxcdata`, `aoxcexec` gibi çekirdek workspaceler, consensus path içinde `aoxcai` çıktısını input olarak kullanmamalıdır.
- `aoxcai` bağımlılığı, consensus execution path'ine eklenmemelidir.
- `aoxcmd` ve operator diagnostics katmanları dışında AI entegrasyonu açılmamalıdır.
- Tüm public docs ve testlerde yukarıdaki anayasal invariant aynen yer almalıdır.

## 2. Hedef mimari: `aoxcunity` scaffold'dan gerçek BFT-kernel'e geçmeli

Bugünkü modül ayrımı (`block`, `vote`, `quorum`, `fork_choice`, `rotation`, `round`, `state`, `validator`, `vote_pool`) iyi bir iskelettir; ancak production-grade bir consensus kernel için aşağıdaki protokol tamamlama adımları zorunludur.

### 2.1 Kimlik doğrulamalı vote modeli

`Vote` yapısı çıplak veri kaydı olmaktan çıkarılıp kriptografik olarak doğrulanabilir canonical message'a dönüştürülmelidir.

#### Zorunlu alanlar

- `network_id` veya `chain_id`
- `era` / `epoch`
- `validator_id`
- `block_hash`
- `height`
- `round`
- `kind`
- `timestamp` veya monotonic logical time
- `validator_set_root`
- `signature_scheme`
- `signature`
- opsiyonel `justification_hash`
- opsiyonel `locked_block_hash`

#### Güvenlik nedeni

Bu model sayesinde:

- vote replay başka ağda kullanılamaz,
- vote başka epoch/era bağlamına taşınamaz,
- vote'un gerçekten yetkili validator tarafından üretildiği doğrulanır,
- finality certificate kurulabilir hale gelir.

#### Uygulama talimatı

- `Vote::signing_bytes()` adında deterministic ve domain-separated bir encoding tanımlanmalıdır.
- Consensus-signing bytes üretimi, serde çıktısına bağımlı olmamalıdır.
- Tüm imza doğrulamaları, admission aşamasında yapılmalıdır.

### 2.2 Equivocation detection zorunlu hale getirilmeli

Aynı validator'ın aynı `(era, height, round, kind)` tuple'ı için iki farklı hedefe oy vermesi açık biçimde tespit edilmelidir.

#### Kurallar

- Her validator için aynı `(era, height, round, kind)` alanında yalnızca tek canonical target olabilir.
- Farklı target'a ikinci vote gelirse:
  - vote reddedilmeli,
  - equivocation evidence üretilmeli,
  - audit/dispute/slashing hattına aktarılmalıdır.
- `Prepare` ve `Commit` için ayrı equivocation indeksleri tutulmalıdır.

#### Uygulama talimatı

`VotePool` içine en az şu yapılar eklenmelidir:

- `votes_by_validator_round`
- `equivocations`
- `latest_vote_by_validator_kind`

Admission sırası:

1. signature + validator eligibility check,
2. equivocation check,
3. duplicate check,
4. persistence/audit append.

### 2.3 Eligibility-safe voting power hesabı

Quorum güvenliği için yalnızca **active + eligible** validator'ların oy gücü sayılmalıdır.

#### Zorunlu API'ler

`ValidatorRotation` veya eşdeğer authority bileşeni şu fonksiyonları sağlamalıdır:

- `eligible_voting_power_of()`
- `eligible_proposer_at()`
- `active_validator_set()`
- `validator_set_root()`
- `contains_active_validator()`

#### Kural seti

- `Observer`, `Inactive`, `Suspended`, `Revoked` üyelerin vote'ları admission aşamasında reddedilmelidir.
- `observed_voting_power()` yalnızca aktif ve vote-eligible validator gücünü saymalıdır.
- `ValidatorRotation::new()` duplicate validator ID'leri reddetmelidir.
- zero voting power politikası açık ve dokümante edilmelidir.
- aktif olmayan ama power sahibi üyeler için kural seti explicit olmalıdır.

## 3. Block admission ve ancestry invariant'ları sertleştirilmeli

`ConsensusState::admit_block()` üretim seviyesi güvenlik için aşağıdaki sıralı doğrulamaları yapmak zorundadır.

### 3.1 Zorunlu admission sırası

1. structural validation,
2. proposer authorization,
3. parent existence,
4. ancestry height continuity,
5. duplicate block hash / duplicate proposal check,
6. epoch/authority-root consistency,
7. fork-choice insertion.

### 3.2 Zorunlu kurallar

- Genesis dışındaki hiçbir block, zero parent ile kabul edilmez.
- `height == parent.height + 1` zorunludur.
- `network_id` eşleşmesi zorunludur.
- `era` ilerleyişi authority policy ile uyumlu olmalıdır.
- proposer, ilgili yükseklik/round için gerçekten yetkili olmalıdır.
- aynı `(height, round, proposer)` için conflicting proposal detection yapılmalıdır.

## 4. Finality, sentetik hash değil gerçek certificate olmalı

Yerel olarak türetilmiş sentetik bir attestation hash, production finality için yeterli değildir. Finality, doğrulanabilir bir `QuorumCertificate` veya eşdeğer seal modeline dayanmalıdır.

### 4.1 Certificate alanları

Minimum certificate yapısı:

- `block_hash`
- `height`
- `round`
- `era`
- `validator_set_root`
- `signers_bitset`
- `aggregated_signature` **veya** signer signatures root
- `observed_voting_power`
- `total_voting_power`
- `threshold_numerator`
- `threshold_denominator`
- `certificate_hash`

### 4.2 Finalize kuralı

`try_finalize()` yalnızca quorum sayısına bakmamalıdır. Finality için şu şartlar birlikte aranmalıdır:

- authenticated commit votes,
- era/height/round eşleşmesi,
- unique eligible signers,
- certificate construction,
- certificate verification,
- finalized ancestor consistency.

## 5. Fork-choice production seviyesine yükseltilmeli

En yüksek finalized ya da en yüksek height tercih etmek tek başına yeterli değildir. Fork-choice güvenliği için justification, locking ve ancestor consistency birlikte korunmalıdır.

### 5.1 Zorunlu kavramlar

`ForkChoice` içinde minimum şu kavramlar yer almalıdır:

- `justified_head`
- `finalized_head`
- `locked_block`
- `best_tip`
- `branch_weight`
- `block_status: Proposed | Prepared | Justified | Finalized | Rejected`

### 5.2 Zorunlu kurallar

- highest justified head önceliği,
- finalized ancestor preference,
- lock-based safety,
- deterministic tie-breaker,
- same-height fork ordering rule,
- ancestor consistency checks,
- rejected branch bookkeeping,
- conflicting finalized branch için panic-level safety assertion.

## 6. Round/pacemaker modeli genişletilmeli

Salt `round: u64` yaklaşımı, production consensus için yetersizdir.

### 6.1 Yeni RoundState

- `current_round`
- `locked_round`
- `highest_prepared_round`
- `highest_justified_round`
- `last_timeout_round`
- `leader_for_round`
- `round_start_time`
- `round_timeout_ms`
- `timeout_certificate_root` veya eşdeğer verified kanıt alanı
- `round_status`

### 6.2 Zorunlu pacemaker akışları

- proposal timeout,
- prepare timeout,
- commit timeout,
- timeout vote,
- timeout certificate,
- round jump recovery.

## 7. Block canonicalization tam ve deterministic olmalı

`BlockHeader` ve `BlockBuilder` tarafında canonical hashing, serde ayrıntılarına değil açıkça tanımlı sıralama kurallarına bağlı olmalıdır.

### 7.1 Gerçek authority root hesabı

`authority_root` mutlaka şu girdilerden türetilmelidir:

- canonical validator ordering,
- validator IDs,
- voting powers,
- active flags,
- `era` / `epoch`,
- authority policy version.

### 7.2 Nested collections canonical ordering

Aşağıdaki kurallar enforce edilmelidir:

- section-per-type tekilliği ya da açık internal ordering,
- lanes by `lane_id`,
- proofs by `(source_network, proof_type, subject_hash)`,
- builder tarafından tüm nested arrays'ın deterministic sıralanması,
- unordered map encoding kullanılmaması,
- enum discriminant'larının domain-separated tanımlanması.

## 8. Persistence ve deterministic recovery zorunlu olmalı

Consensus state yalnızca RAM'de tutulamaz. Kernel seviyesinde dayanıklı storage ve deterministic replay gereklidir.

### 8.1 Zorunlu store ayrımı

- `ConsensusState`: in-memory working set
- `ConsensusStore`: durable backend trait
- restore path: deterministic replay only

### 8.2 Zorunlu depolama bileşenleri

- block headers,
- block bodies,
- vote log,
- equivocation evidence,
- quorum certificates,
- finalized checkpoints,
- validator-set snapshots,
- pacemaker state,
- replay cursor.

### 8.3 Recovery kuralları

- restart sonrası son finalized block'tan recover,
- unfinalized branch replay,
- durable vote deduplication,
- half-written state detection,
- checksum/corruption detection.

## 9. `aoxcore` authority source of truth olmalı

Validator authority, consensus motoru içinde ad hoc üretilmemeli; immutable authority snapshot olarak yukarı katmandan gelmelidir.

### 9.1 Sorumluluk ayrımı

`aoxcore`:

- genesis authority source,
- validator set governance,
- validator revocation,
- era transition authorization.

`aoxcunity`:

- authority verisini consume eder,
- consensus execution yapar,
- certificate üretir,
- authority policy üretmez.

### 9.2 Consume edilecek immutable yapı

En az şu alanları içeren bir `AuthoritySnapshot` veya eşdeğeri kullanılmalıdır:

- `era`
- `validator_set`
- `authority_root`
- `activation_height`
- `deactivation_rules`

## 10. `aoxcnet` ile authenticated transport envelope kurulmalı

Çıplak consensus message enum'ı, hostile network ortamı için yeterli değildir.

### 10.1 Zorunlu envelope alanları

- `network_id`
- `era`
- `sender_id`
- `message_type`
- `message_hash`
- `signature`
- `timestamp` veya monotonic sequence
- opsiyonel compression/version alanları

### 10.2 Güvenlik gereksinimleri

- peer authentication,
- message signature verification,
- replay window,
- rate limiting,
- peer scoring,
- equivocation propagation,
- finalize certificate dissemination,
- epoch-aware disconnect rules.

## 11. Audit-grade test stratejisi

Bu alanda yalnızca happy-path unit test kabul edilmemelidir. Consensus güvenliği için çok katmanlı test matrisi zorunludur.

### 11.1 Unit tests

Minimum zorunlu testler:

- invalid parent rejection,
- parent-height mismatch rejection,
- inactive validator vote rejection,
- observer vote rejection,
- duplicate validator set rejection,
- zero-power policy validation,
- duplicate proposal rejection,
- same-round equivocation rejection,
- wrong-era vote rejection,
- wrong-network vote rejection.

### 11.2 Property tests

- quorum monotonicity,
- canonical hash determinism,
- certificate determinism,
- vote pool idempotence,
- replay determinism,
- finalized chain ancestor invariant.

### 11.3 Stateful / model tests

- byzantine double-vote scenario,
- conflicting forks across equal height,
- restart + replay + finality consistency,
- network partition then recovery,
- delayed certificate arrival,
- proposer failure then pacemaker recovery.

### 11.4 Fuzz tests

- malformed votes,
- malformed block bodies,
- random message ordering,
- corrupted store replay,
- hostile transport envelope fields.

### 11.5 Uygulama talimatı

- `proptest` tabanlı property testler eklenmelidir.
- consensus state machine için model-based veya state-machine testing kurulmalıdır.
- fuzz hedefleri CI dışında da düzenli olarak koşturulmalıdır.
- replay ve persistence davranışı crash senaryoları ile birlikte doğrulanmalıdır.

## 12. Formal invariant dokümanı zorunlu

Kernel-grade bir sistem için yalnızca kod değil, audit-friendly invariant dokümanı da gerekir.

### 12.1 Minimum invariant set

- AI never influences consensus truth.
- Only authenticated eligible validators may vote.
- A validator cannot cast two conflicting votes for the same `(era, height, round, kind)`.
- A block must reference an existing parent except genesis.
- Child height must equal parent height + 1.
- Finality certificate must bind block, era, round, validator set, and signer set.
- Finalized blocks must form a single ancestor-consistent chain.
- Restart and replay must preserve finalized state exactly.
- Authority root in block header must match active validator snapshot.
- Fork choice must never prefer a non-ancestor over a finalized head.

### 12.2 Her invariant için zorunlu belge alanları

- prose definition,
- enforcement location,
- test names,
- failure mode,
- audit rationale.

## 13. Önceliklendirilmiş uygulama yol haritası

### 13.1 Must fix before merge

- eligibility-safe voting power accounting,
- vote signatures,
- equivocation detection,
- strict parent/height checks,
- gerçek authority root hesabı,
- canonical nested ordering,
- yukarıdaki maddeler için eksiksiz unit tests.

### 13.2 Must fix before testnet

- quorum certificate modeli,
- validator-set snapshots,
- persistent consensus store,
- replay/recovery,
- authenticated transport envelope,
- pacemaker expansion,
- property tests + fuzzing.

### 13.3 Must fix before mainnet

- slashing/dispute evidence pipeline,
- hostile network simulation,
- crash consistency tests,
- formal invariant spec,
- storage corruption recovery,
- DoS hardening/performance caps,
- external audit trail completeness,
- deterministic upgrade/version migration policy.

## 14. Codex'e verilecek kısa ve net görev tanımı

Aşağıdaki görev tanımı, repo içinde uygulanacak refactor işi için doğrudan kullanılabilir:

> Refactor AOXC core consensus toward a production-grade kernel architecture.
>
> Primary goals:
> 1. Keep `aoxcai` strictly non-authoritative and operator-plane only.
> 2. Upgrade `aoxcunity` from a deterministic scaffold into a cryptographically authenticated, safety-preserving BFT-style consensus core.
> 3. Preserve deterministic behavior, panic-free error handling, and audit-grade documentation.
> 4. Do not introduce dead code, placeholder code, or empty abstractions.
>
> Mandatory architectural constraints:
> - AI must never influence consensus truth, fork choice, quorum, finality, validator authority, or state transition validity.
> - All votes must be authenticated, era-bound, network-bound, and validator-set-bound.
> - Equivocation must be explicitly detected and surfaced as evidence.
> - Quorum counting must include only eligible active validators.
> - Finality must require a verifiable quorum certificate, not a synthetic local hash.
> - Block admission must enforce parent existence, exact height continuity, proposer eligibility, and authority-root consistency.
> - Consensus state must support durable persistence and deterministic replay.
> - All canonical hashing and signing bytes must be domain-separated and independent from serde encoding details.
>
> Implementation phases:
> - Phase 1: safety-critical repairs.
> - Phase 2: authenticated vote and certificate model.
> - Phase 3: fork-choice and pacemaker hardening.
> - Phase 4: persistence and recovery.
> - Phase 5: transport authentication and replay protection.
> - Phase 6: property tests, fuzz tests, and invariant documentation.

## 15. Son hüküm

- `aoxcai` korunmalı ve genişletilmeli; ancak sınırları asla delinmemelidir.
- `aoxcunity`, imzalı, deterministik, replayable ve certificate-backed bir consensus kernel'e dönüşmelidir.
- AI operator intelligence plane olarak kalmalı; consensus truth ise yalnızca deterministic kernel kuralları ve authenticated validator authority tarafından belirlenmelidir.
