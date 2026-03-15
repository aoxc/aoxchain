# AOXChain Derin Teknik Analiz (TR)

## 1) Sistemi anladım mı? (Kısa cevap)
Evet. Repo, çok-crate (workspace) mimarisinde bir L1/L0 zincir çekirdeği kurmaya çalışıyor:
- `aoxcore`: kimlik, genesis, mempool, transaction ve temel zincir domain’i.
- `aoxcunity`: consensus/fork-choice/validator rotation katmanı.
- `aoxcvm`: çok-lane VM yürütme (EVM, WASM, Sui Move, Cardano).
- `aoxcrpc`: HTTP/gRPC/WebSocket erişim katmanı.
- `aoxcnet`: p2p/gossip/discovery/sync.
- `aoxcmd`: node orkestrasyonu/çalıştırma katmanı.

Bu ayrım doğru yönde; ancak katman sözleşmeleri (API kontratları) tam senkron değil.

## 2) Mimari güçlü taraflar
1. **Domain ayrışması iyi:** consensus, VM, network, RPC, core ayrı crate’lerde.
2. **Workspace standardizasyonu var:** ortak dependency yönetimi (`workspace.dependencies`).
3. **Consensus çekirdeği tutarlı:** `aoxcunity` içinde validator, quorum, vote pool ve fork-choice modeli net.
4. **VM lane yaklaşımı vizyoner:** farklı execution model’leri tek host/routing altında düşünülmüş.

## 3) Kritik problemler (öncelik sırasıyla)

### P0 — Workspace derlenmiyor
Tüm proje için `cargo test` başarısız. Ana kırılım `aoxcmd` katmanında.

Öne çıkan kök nedenler:
1. **Yanlış crate adı kullanımı:** `aoxcmd` kodu `aoxcore` import ediyor; workspace’de crate adı `aoxccore`.
2. **Eksik dependency bildirimi:** `aoxcmd` içinde kullanılan `serde_json` ve `hex`, `Cargo.toml`’da yok.
3. **Eski API’lere bağlı kod:** `aoxcmd/node/state.rs`, `aoxcunity` güncel tip imzalarıyla uyuşmuyor.
   - `ConsensusState::new()` çağrısı argümansız; güncel imza argüman istiyor.
   - `ValidatorRotation::new(&validators)` yanlış tipte çağrılıyor.
   - `Validator` alanları (`actor_id`, `public_key`) artık yok; yeni model `id`, `voting_power`, `active`.
4. **Engine dosyası da sürüklenmiş durumda:** `aoxcmd/node/engine.rs` içinde `aoxcunity::block_builder::BlockBuilder` import’u geçersiz görünüyor.

### P0 — Katmanlar arası kontrat drift’i
`aoxcmd`, consensus çekirdeğine “eski interface” üzerinden konuşuyor. Bu, hızlı iterasyonlarda normal ama şu an compile barrier oluşturuyor.

### P1 — Dokümantasyon yetersizliği
`README.md` pratikte boş. Yeni contributor için “nasıl ayağa kaldırılır / hangi crate ne yapar / hangi crate production-ready” görünmüyor.

### P1 — Test stratejisi parçalı
Bazı crate’ler derlenirken, workspace top-level pipeline kırılıyor. Bu, CI’da “kısmi yeşil” yanılsaması üretir.

### P2 — Operasyonel olgunluk sinyalleri eksik
- Deterministic integration test matrisi (core+consensus+cmd) görünmüyor.
- Release gate/policy dokümante değil.

## 4) Teknik tespitlerin kanıt haritası

1. **Workspace ve crate listesi**: kökte çoklu crate yapısı mevcut.
2. **`aoxcmd` dependency seti yetersiz**: yalnızca birkaç crate tanımlı.
3. **`aoxcmd` state bootstrap kodu, güncel consensus API ile uyumsuz**.
4. **`aoxcunity` tarafında `ConsensusState::new(rotation, quorum)` imzası var**.
5. **`aoxcunity::Validator` alanları güncel modelde farklı**.
6. **`cargo test` çıktısı bu uyumsuzlukları doğrudan hata olarak veriyor**.

## 5) Önerilen iyileştirme planı (uygulanabilir)

### Faz 1 — Derlenebilirlik (1-2 gün)
1. `aoxcmd` importlarını crate adıyla hizala (`aoxccore`).
2. `aoxcmd/Cargo.toml` içine kullanılan crate’leri ekle (`serde_json`, `hex`, gerekiyorsa `aoxcnet` vs.).
3. `node/state.rs` bootstrap akışını `aoxcunity` güncel API’sine göre refactor et:
   - validator üretimi `Validator::new(...)`
   - rotation oluşturma sonucu `Result` ise uygun hata akışına bağla
   - quorum ve consensus init parametrelerini explicit ver
4. `node/engine.rs` importlarını ve `AOXCNode` alan kullanımını güncelle.

### Faz 2 — Kontrat sertleştirme (2-4 gün)
1. `aoxcmd <-> aoxcunity` için bir adapter/modül sınırı tanımla.
2. “Breaking change checklist” ekle (consensus API değişirse cmd katmanı otomatik fail etsin).
3. En az bir integration test: `node setup -> 1 block proposal -> vote -> finalize`.

### Faz 3 — Üretim hazırlığı (1 hafta)
1. README’yi “quickstart + architecture map + current limitations” ile doldur.
2. CI pipeline’ı iki seviyeye ayır:
   - fast lane: `cargo check --workspace`
   - deep lane: `cargo test --workspace`
3. Versioning policy: crate API stability seviyeleri (`experimental`, `stable-core`) ekle.

## 6) Öncelikli risk matrisi
- **Yüksek etki / yüksek olasılık:** compile kırıkları nedeniyle yeni feature teslimleri bloklanır.
- **Yüksek etki / orta olasılık:** consensus-node orchestration uyuşmazlığı runtime’da split-brain benzeri davranışlara yol açabilir.
- **Orta etki / yüksek olasılık:** dokümantasyon eksikliği onboarding süresini ve hata oranını artırır.

## 7) Sonuç (net değerlendirme)
Proje fikri ve modüler tasarım güçlü. En büyük eksik “teknik vizyon” değil, **entegrasyon disiplini**:
- API kontratları süratli değişmiş,
- orchestration katmanı geride kalmış,
- workspace derlenebilirliği kırılmış.

Kısa vadede hedef net olmalı: **önce tek komutta derlenen workspace**, sonra feature geliştirme.
